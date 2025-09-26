use async_trait::async_trait;
use aws_sdk_sesv2::{Client as SesClient, Error as SesError};
use domain::entities::*;
use domain::events::*;
use domain::error::{DomainError, Result};
use application::ports::EmailService;
use std::sync::Arc;

use crate::config::Config;

/// SES-based email service implementation
pub struct SesEmailService {
    client: SesClient,
    config: Arc<Config>,
    from_email: String,
}

impl SesEmailService {
    pub fn new(client: SesClient, config: Arc<Config>, from_email: String) -> Self {
        Self {
            client,
            config,
            from_email,
        }
    }
}

#[async_trait]
impl EmailService for SesEmailService {
    async fn send_rfq_created_notification(&self, rfq: &RfqMeta) -> Result<()> {
        // Send notification to manufacturer
        let manufacturer_subject = format!("New RFQ: {}", rfq.subject);
        let manufacturer_body = format!(
            "Hello,\n\n\
            You have received a new Request for Quote (RFQ).\n\n\
            Subject: {}\n\
            From: {} ({})\n\n\
            Please log in to your account to view the details and respond.\n\n\
            Best regards,\n\
            Terra Platform",
            rfq.subject,
            rfq.buyer.name.as_ref().unwrap_or(&"Anonymous".to_string()),
            rfq.buyer.email
        );

        // Find manufacturer email from participants
        let manufacturer_email = rfq.participants.iter()
            .find(|p| p.role == ParticipantRole::Manufacturer)
            .map(|p| &p.email);

        if let Some(to_email) = manufacturer_email {
            self.send_email(to_email, &manufacturer_subject, &manufacturer_body).await?;
        }

        // Send confirmation to buyer
        let buyer_subject = "RFQ Submitted Successfully";
        let buyer_body = format!(
            "Hello {},\n\n\
            Your Request for Quote has been submitted successfully.\n\n\
            Subject: {}\n\
            RFQ ID: {}\n\n\
            The manufacturer will be notified and should respond within a few business days.\n\
            You will receive notifications for any updates.\n\n\
            Best regards,\n\
            Terra Platform",
            rfq.buyer.name.as_ref().unwrap_or(&"Customer".to_string()),
            rfq.subject,
            rfq.id
        );

        self.send_email(&rfq.buyer.email, buyer_subject, &buyer_body).await?;

        Ok(())
    }

    async fn send_rfq_message_notification(&self, rfq: &RfqMeta, event: &RfqEvent) -> Result<()> {
        if let RfqEvent::Message(message_event) = event {
            let (to_email, from_role) = match message_event.base.by {
                EventAuthor::Buyer => {
                    // Message from buyer, notify manufacturer
                    let manufacturer_email = rfq.participants.iter()
                        .find(|p| p.role == ParticipantRole::Manufacturer)
                        .map(|p| &p.email);
                    (manufacturer_email, "buyer")
                }
                EventAuthor::Manufacturer => {
                    // Message from manufacturer, notify buyer
                    (Some(&rfq.buyer.email), "manufacturer")
                }
                EventAuthor::System => {
                    // System messages don't trigger notifications
                    return Ok(());
                }
            };

            if let Some(recipient_email) = to_email {
                let subject = format!("New message on RFQ: {}", rfq.subject);
                let body = format!(
                    "Hello,\n\n\
                    You have received a new message on your RFQ.\n\n\
                    Subject: {}\n\
                    RFQ ID: {}\n\
                    From: {}\n\n\
                    Message:\n\
                    {}\n\n\
                    Please log in to your account to view the full conversation and respond.\n\n\
                    Best regards,\n\
                    Terra Platform",
                    rfq.subject,
                    rfq.id,
                    from_role,
                    message_event.body
                );

                self.send_email(recipient_email, &subject, &body).await?;
            }
        }

        Ok(())
    }
}

impl SesEmailService {
    async fn send_email(&self, to_email: &str, subject: &str, body: &str) -> Result<()> {
        let content = aws_sdk_sesv2::types::EmailContent::builder()
            .simple(
                aws_sdk_sesv2::types::Message::builder()
                    .subject(
                        aws_sdk_sesv2::types::Content::builder()
                            .data(subject)
                            .charset("UTF-8")
                            .build()
                            .map_err(|e| DomainError::Internal(format!("Failed to build subject: {}", e)))?,
                    )
                    .body(
                        aws_sdk_sesv2::types::Body::builder()
                            .text(
                                aws_sdk_sesv2::types::Content::builder()
                                    .data(body)
                                    .charset("UTF-8")
                                    .build()
                                    .map_err(|e| DomainError::Internal(format!("Failed to build body: {}", e)))?,
                            )
                            .build(),
                    )
                    .build()
                    .map_err(|e| DomainError::Internal(format!("Failed to build message: {}", e)))?,
            )
            .build();

        let destination = aws_sdk_sesv2::types::Destination::builder()
            .to_addresses(to_email)
            .build();

        self.client
            .send_email()
            .from_email_address(&self.from_email)
            .destination(destination)
            .content(content)
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to send email: {}", e)))?;

        Ok(())
    }
}
use jsonschema::{JSONSchema, ValidationError};
use serde_json::Value;

/// JSON schema validation utility
pub struct JsonValidator {
    schema: JSONSchema,
}

impl JsonValidator {
    pub fn new(schema_value: Value) -> Result<Self, String> {
        let schema = JSONSchema::compile(&schema_value)
            .map_err(|e| format!("Failed to compile schema: {}", e))?;
        
        Ok(Self { schema })
    }
    
    pub fn validate(&self, instance: &Value) -> Result<(), Vec<ValidationError>> {
        let result = self.schema.validate(instance);
        
        match result {
            Ok(_) => Ok(()),
            Err(errors) => Err(errors.collect()),
        }
    }
}

/// Common validation schemas as per the design document
pub fn create_rfq_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "required": ["tenant_id", "manufacturer_id", "buyer", "subject", "body"],
        "properties": {
            "tenant_id": {
                "type": "string",
                "pattern": "^[a-zA-Z0-9_-]+$",
                "minLength": 1,
                "maxLength": 50
            },
            "manufacturer_id": {
                "type": "string",
                "pattern": "^mfg_[a-zA-Z0-9_-]+$",
                "minLength": 5,
                "maxLength": 50
            },
            "buyer": {
                "type": "object",
                "required": ["email"],
                "properties": {
                    "email": {
                        "type": "string",
                        "format": "email"
                    },
                    "name": {
                        "type": "string",
                        "maxLength": 200
                    }
                }
            },
            "subject": {
                "type": "string",
                "minLength": 1,
                "maxLength": 200
            },
            "body": {
                "type": "string",
                "minLength": 1,
                "maxLength": 8000
            },
            "attachments": {
                "type": "array",
                "maxItems": 10,
                "items": {
                    "type": "object",
                    "required": ["upload_key", "file_name", "content_type", "size_bytes"],
                    "properties": {
                        "upload_key": {
                            "type": "string",
                            "minLength": 1
                        },
                        "file_name": {
                            "type": "string",
                            "minLength": 1,
                            "maxLength": 255
                        },
                        "content_type": {
                            "type": "string",
                            "enum": ["image/jpeg", "image/png", "image/webp", "image/avif", "application/pdf"]
                        },
                        "size_bytes": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 15728640
                        }
                    }
                }
            }
        },
        "additionalProperties": false
    })
}

pub fn post_message_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "required": ["by", "body"],
        "properties": {
            "by": {
                "type": "string",
                "enum": ["buyer", "manufacturer"]
            },
            "body": {
                "type": "string",
                "minLength": 1,
                "maxLength": 8000
            },
            "attachments": {
                "type": "array",
                "maxItems": 10,
                "items": {
                    "type": "object",
                    "required": ["upload_key", "file_name", "content_type", "size_bytes"],
                    "properties": {
                        "upload_key": {
                            "type": "string",
                            "minLength": 1
                        },
                        "file_name": {
                            "type": "string",
                            "minLength": 1,
                            "maxLength": 255
                        },
                        "content_type": {
                            "type": "string",
                            "enum": ["image/jpeg", "image/png", "image/webp", "image/avif", "application/pdf"]
                        },
                        "size_bytes": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 15728640
                        }
                    }
                }
            }
        },
        "additionalProperties": false
    })
}

pub fn presign_upload_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "required": ["tenant_id", "pathType", "content_type", "size_bytes"],
        "properties": {
            "tenant_id": {
                "type": "string",
                "pattern": "^[a-zA-Z0-9_-]+$",
                "minLength": 1,
                "maxLength": 50
            },
            "pathType": {
                "type": "string",
                "enum": ["imageRaw"]
            },
            "content_type": {
                "type": "string",
                "enum": ["image/jpeg", "image/png", "image/webp", "image/avif", "application/pdf"]
            },
            "size_bytes": {
                "type": "integer",
                "minimum": 1,
                "maximum": 15728640
            }
        },
        "additionalProperties": false
    })
}
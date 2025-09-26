<script>
  export let items = [];
  
  let filteredItems = items;
  let searchTerm = '';
  let selectedState = '';
  
  $: {
    filteredItems = items.filter(item => {
      const matchesSearch = !searchTerm || 
        item.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        item.city.toLowerCase().includes(searchTerm.toLowerCase());
      
      const matchesState = !selectedState || item.state === selectedState;
      
      return matchesSearch && matchesState;
    });
  }
  
  // Get unique states for filter dropdown
  $: uniqueStates = [...new Set(items.map(item => item.state))].sort();
</script>

<div class="filter-controls">
  <div class="filter-row">
    <input
      type="text"
      placeholder="Search manufacturers..."
      bind:value={searchTerm}
      class="search-input"
    />
    
    <select bind:value={selectedState} class="state-filter">
      <option value="">All States</option>
      {#each uniqueStates as state}
        <option value={state}>{state}</option>
      {/each}
    </select>
  </div>
  
  <p class="results-count">
    Showing {filteredItems.length} of {items.length} manufacturers
  </p>
</div>

<div class="grid" id="filtered-results">
  {#each filteredItems as manufacturer}
    <article class="card manufacturer-card">
      {#if manufacturer.logoUrl}
        <img 
          src={manufacturer.logoUrl} 
          alt={manufacturer.name}
          class="manufacturer-logo"
          loading="lazy"
        />
      {/if}
      <div class="manufacturer-info">
        <h4>
          <a href="/catalog/manufacturer/{manufacturer.id}/">{manufacturer.name}</a>
        </h4>
        <p>{manufacturer.city}, {manufacturer.state}</p>
        {#if manufacturer.capabilities}
          <p class="capabilities">{manufacturer.capabilities.join(', ')}</p>
        {/if}
        <div style="margin-top: 1rem;">
          <a href="/catalog/manufacturer/{manufacturer.id}/" class="btn">View Details</a>
          <a href="/rfq/submit?mfg={manufacturer.id}" class="btn btn-primary" style="margin-left: 0.5rem;">
            Submit RFQ
          </a>
        </div>
      </div>
    </article>
  {/each}
  
  {#if filteredItems.length === 0}
    <div class="no-results">
      <h3>No manufacturers found</h3>
      <p>Try adjusting your search or filter criteria.</p>
    </div>
  {/if}
</div>

<style>
  .filter-controls {
    background: white;
    padding: 1.5rem;
    border-radius: 12px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.08);
    margin-bottom: 2rem;
  }
  
  .filter-row {
    display: flex;
    gap: 1rem;
    margin-bottom: 1rem;
  }
  
  .search-input, .state-filter {
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
  }
  
  .search-input {
    flex: 1;
  }
  
  .state-filter {
    min-width: 150px;
  }
  
  .results-count {
    color: #7f8c8d;
    font-size: 0.9rem;
  }
  
  .capabilities {
    font-size: 0.9rem;
    color: #3498db;
    font-weight: 500;
  }
  
  .no-results {
    grid-column: 1 / -1;
    text-align: center;
    padding: 3rem;
    color: #7f8c8d;
  }
  
  .manufacturer-card {
    display: block;
    text-decoration: none;
    color: inherit;
  }
  
  .manufacturer-card h4 a {
    color: #2c3e50;
    text-decoration: none;
  }
  
  .manufacturer-card h4 a:hover {
    color: #3498db;
  }
</style>
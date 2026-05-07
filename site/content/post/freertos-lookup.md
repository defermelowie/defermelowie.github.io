+++
title = "FreeRTOS name lookup"
date = "2026-5-6"
[taxonomies]
tags = ["freertos"]
+++

<input type="text" id="search-field">
<div id="suggestions-container"></div>
<script type="module">
import init, { search } from '/freertos_lookup.js';
await init();
document.getElementById('search-field').addEventListener('input', (event) => {
    const query = event.target.value;
    console.info(`Searching for ${query}`);
    const suggestions = search(query);
    const suggestionsContainer = document.getElementById('suggestions-container');
    suggestionsContainer.innerHTML = '';
    suggestions.forEach(suggestion => {
        const div = document.createElement('div');
        div.textContent = suggestion;
        suggestionsContainer.appendChild(div);
    });
});
</script>

+++
title = "FreeRTOS name lookup"
date = "2026-5-6"
[taxonomies]
tags = ["freertos"]
+++

<input type="text" id="search-field">
<script type="module">
import init, { search } from '/freertos_lookup.js';
await init();
document.getElementById('search-field').addEventListener('input', (event) => {
    const query = event.target.value;
    console.info(`Searching for ${query}`);
    search(query);
});
</script>

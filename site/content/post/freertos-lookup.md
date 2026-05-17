+++
title = "FreeRTOS name lookup"
date = "2026-5-6"
[taxonomies]
tags = ["freertos"]
+++

<style>
    .centered {
        display: flex;
        flex-direction: column;
        align-items: center;
    }
   #search-field {
       width: 100%;
       max-width: 600px;
       font-size: 1rem;
       padding: 8px;
       border: 1px solid #ccc;
       border-radius: 4px;
   }
   #suggestions-container {
       margin-top: 10px;
       cursor: pointer;
   }
</style>

<!-- Start of content -->

<div class="centered">

Enter a FreeRTOS function name to learn more about it

<input type="text" id="search-field" style="width:60%;">
<div id="suggestions-container"></div>
    
</div>

<!-- End of content -->

<script type="module">
    import init, { search } from '/freertos_lookup.js';
    await init();
    const searchField = document.getElementById('search-field');
    searchField.addEventListener('input', (event) => {
        const query = event.target.value;
        const suggestions = search(query);
        console.info(`Searching for ${query} yields ${suggestions}`);
        
        const suggestionsContainer = document.getElementById('suggestions-container');
        suggestionsContainer.innerHTML = '';
        suggestions.forEach(suggestion => {
            const div = document.createElement('div');
            div.textContent = suggestion;
            div.onclick = () => {
                searchField.value = suggestion;
                suggestionsContainer.innerHTML = '';
                console.info(`Selected suggestion: ${suggestion}`);
            };
            suggestionsContainer.appendChild(div);
        });
    });
</script>

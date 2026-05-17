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
        margin: 0 auto;
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

   .suggestion-item {
       padding: 8px 10px;
       cursor: pointer;
   }
   
   .suggestion-item.selected {
       background-color: #e9e9e9;
   }
</style>

<!-- Start of content -->

<div id="search-container" class="centered">
    <label for="search-field">Enter a FreeRTOS function name to learn more about it</label>
    <input type="text" id="search-field" style="width:60%;">
    <div id="suggestions-container" role="listbox" aria-labelledby="search-field"></div>
</div>

<!-- End of content -->

<script type="module">
    import init, { search } from '/freertos_lookup.js';
    await init();
    
    const searchField = document.getElementById('search-field');
    const suggestionsContainer = document.getElementById('suggestions-container');
    let selectedSuggestionIdx = -1;

    /** 
     * Populate suggestion container based on the current search query
     * @param {string[]} query
     */
     function showSuggestions(query) {
         const suggestions = search(query, 3);
         suggestionsContainer.innerHTML = '';
         selectedSuggestionIdx = 0;

        // Log an error if suggestions is empty
        if (suggestions.length === 0) {
            console.error(`Error: No suggestions returned for query "${query}". This should not happen as "search(query)" should always return at least the query itself.`);
            return;
        }

        // If there's only one suggestion, return early
        if (suggestions.length === 1) {
            return;
        }

        // Populate suggestions container
        suggestions.forEach((suggestion, idx) => {
            const div = document.createElement('div');
            div.textContent = suggestion;
            div.classList.add('suggestion-item');
            div.setAttribute('role', 'option');

            // Click event to select a suggestion
            div.onclick = () => {
                selectedSuggestionIdx = idx;
                commitSelectedSuggestion();
            };

            // Hover event to highlight a suggestion
            div.onmouseover = () => {
                selectedSuggestionIdx = idx;
                updateSelectedSuggestion(suggestionsContainer.querySelectorAll('.suggestion-item'));
            };

            suggestionsContainer.appendChild(div);
        });

        // Highlight selected suggestion
        updateSelectedSuggestion(suggestionsContainer.querySelectorAll('.suggestion-item'));
     }

    // Update the selected suggestion's appearance
    function updateSelectedSuggestion(suggestions) {
        suggestions.forEach((suggestion, index) => {
            if (index === selectedSuggestionIdx) {
                suggestion.classList.add('selected');
                suggestion.setAttribute('aria-selected', 'true');
            } else {
                suggestion.classList.remove('selected');
                suggestion.setAttribute('aria-selected', 'false');
            }
        });
    }

    // Commit the selected a suggestion & close others
    function commitSelectedSuggestion() {
        let suggestion = suggestionsContainer.querySelectorAll('.suggestion-item')[selectedSuggestionIdx].textContent;
        searchField.value = suggestion;
        suggestionsContainer.innerHTML = '';
        selectedSuggestionIdx = -1;
    }

    // Handle input event
    searchField.addEventListener('input', (event) => {
        const query = event.target.value;
        showSuggestions(query);
    });

    // Handle keyboard navigation
    searchField.addEventListener('keydown', (event) => {
        const suggestions = suggestionsContainer.querySelectorAll('.suggestion-item');
        const suggestionCount = suggestions.length;
        
        if (suggestionCount === 1) {
            return;
        }

        switch(event.key){
            case 'ArrowDown':
                event.preventDefault();
                selectedSuggestionIdx = (selectedSuggestionIdx + 1) % suggestionCount;
            break;
            case 'ArrowUp':
                event.preventDefault();
                selectedSuggestionIdx = (selectedSuggestionIdx - 1) % suggestionCount;
            break;
            case 'Enter':
                event.preventDefault();
                commitSelectedSuggestion();
            break;
            case 'Escape':
                event.preventDefault();
                suggestionsContainer.innerHTML = '';
                selectedSuggestionIdx = -1;
                break;
        }

        updateSelectedSuggestion(suggestions);
    });
</script>

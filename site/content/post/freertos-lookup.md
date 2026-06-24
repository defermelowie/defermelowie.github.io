+++
title = "FreeRTOS name lookup"
date = "2026-5-6"
[taxonomies]
tags = ["freertos"]
+++

{{ searchbar(id="ident" label="Enter a FreeRTOS identifier to learn more about it") }}

<div id="info-container"></div>

# FreeRTOS naming convention

The hungarian naming scheme used by FreeRTOS can be represented as follows:

```ebnf
identifier   = typed_ident | prv_function | macro ;

typed_ident  = type, name ;
prv_function = "prv", name ;
macro        = ? C identifier: lowercase module prefix, UPPER_SNAKE_CASE body ? ;

type         = ["p"], ["u"], base_type ;
base_type    = "l" | "s" | "c" | "x" | "e" | "v" ;
name         = ? PascalCase C identifier ? ;
```

Note that `typed_ident` covers both variables and API functions &mdash; they are syntactically identical and cannot be distinguished by name alone.
Instead, this lookup tool allows to make a distinction by putting parenthesis after a function identifier.

<!-- End of content -->

<script type="module">
    import init, { search, type_of, ident_of, kind_of, doc_url } from '/freertos_lookup.js';
    await init();
    
    const searchField = document.getElementById('search-ident-field');
    const infoContainer = document.getElementById('info-container');
    const suggestionsContainer = document.getElementById('suggest-ident-container');
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

     function showInfo(name) {
         infoContainer.innerHTML = '';

         const nameContainer = document.createElement('h1');
         nameContainer.innerText = kind_of(name).charAt(0).toUpperCase() + kind_of(name).slice(1) + " " + ident_of(name);
         infoContainer.appendChild(nameContainer);
         
         const typeContainer = document.createElement('p');
         typeContainer.innerText = 'type: ' + type_of(name);
         infoContainer.appendChild(typeContainer);

         const doc_link = doc_url(name);
         if (typeof doc_link === 'string') {
             const docContainer = document.createElement('a');
             docContainer.href = doc_link;
             docContainer.textContent = 'View Documentation';
             docContainer.target = '_blank';
             infoContainer.appendChild(docContainer);
         }
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

        showInfo(suggestion);
    }

    // Handle input event
    searchField.addEventListener('input', (event) => {
        const query = event.target.value;
        showSuggestions(query);
        showInfo(query);
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

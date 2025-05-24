/**
 * Code copy functionality for Sphinx documentation
 * Adds a copy button to all code blocks
 */

document.addEventListener('DOMContentLoaded', function() {
    // Find all code blocks
    const codeBlocks = document.querySelectorAll('div.highlight pre, pre.literal-block');
    
    codeBlocks.forEach(function(codeBlock) {
        // Skip if already has a copy button
        if (codeBlock.parentNode.querySelector('.code-copy-button')) {
            return;
        }
        
        // Create copy button
        const button = document.createElement('button');
        button.className = 'code-copy-button';
        button.innerHTML = '<i class="fa fa-copy"></i> Copy';
        button.setAttribute('aria-label', 'Copy code to clipboard');
        
        // Position the button
        const wrapper = codeBlock.parentNode;
        wrapper.style.position = 'relative';
        wrapper.appendChild(button);
        
        // Copy functionality
        button.addEventListener('click', async function() {
            let code = codeBlock.textContent || codeBlock.innerText;
            
            // Remove line numbers if present
            const linenosElement = wrapper.querySelector('.linenos');
            if (linenosElement) {
                // Find the actual code content (not line numbers)
                const codeContent = wrapper.querySelector('.highlight > pre');
                if (codeContent) {
                    // Clone the node to avoid modifying the DOM
                    const tempNode = codeContent.cloneNode(true);
                    // Remove any line number elements
                    const linenosInCode = tempNode.querySelector('.linenos');
                    if (linenosInCode) {
                        linenosInCode.remove();
                    }
                    code = tempNode.textContent || tempNode.innerText;
                } else {
                    // Fallback: remove line numbers manually
                    const lines = code.split('\n');
                    code = lines.map(line => {
                        // Remove leading line numbers (e.g., "  1 " or " 42 ")
                        return line.replace(/^\s*\d+\s+/, '');
                    }).join('\n');
                }
            }
            
            try {
                if (navigator.clipboard && window.isSecureContext) {
                    // Use the modern clipboard API
                    await navigator.clipboard.writeText(code);
                } else {
                    // Fallback for older browsers
                    const textArea = document.createElement('textarea');
                    textArea.value = code;
                    textArea.style.position = 'fixed';
                    textArea.style.left = '-999999px';
                    document.body.appendChild(textArea);
                    textArea.select();
                    document.execCommand('copy');
                    document.body.removeChild(textArea);
                }
                
                // Show success feedback
                button.innerHTML = '<i class="fa fa-check"></i> Copied!';
                button.classList.add('success');
                
                // Reset button after 2 seconds
                setTimeout(function() {
                    button.innerHTML = '<i class="fa fa-copy"></i> Copy';
                    button.classList.remove('success');
                }, 2000);
                
            } catch (err) {
                console.error('Failed to copy code:', err);
                button.innerHTML = '<i class="fa fa-times"></i> Failed';
                button.classList.add('error');
                
                setTimeout(function() {
                    button.innerHTML = '<i class="fa fa-copy"></i> Copy';
                    button.classList.remove('error');
                }, 2000);
            }
        });
    });
});

// Also handle dynamically loaded content (e.g., from tabs)
if (typeof MutationObserver !== 'undefined') {
    const observer = new MutationObserver(function(mutations) {
        mutations.forEach(function(mutation) {
            if (mutation.addedNodes.length) {
                mutation.addedNodes.forEach(function(node) {
                    if (node.nodeType === 1) { // Element node
                        const codeBlocks = node.querySelectorAll('div.highlight pre, pre.literal-block');
                        if (codeBlocks.length > 0) {
                            // Re-run the initialization
                            document.dispatchEvent(new Event('DOMContentLoaded'));
                        }
                    }
                });
            }
        });
    });
    
    observer.observe(document.body, {
        childList: true,
        subtree: true
    });
}
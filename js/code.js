//
// This JavaScript code is the full version of the minified
// JavaScript included in the library code for the POST method.
//
// reScript function is a trick to make the Javascript code work when inserted.
// Replace it with another clone element script.
//
(function () {
    async function fetchShortcodeData() {
        try {
            const request = new Request("{-- replaced with url --}", {
                headers: (() => {
                    const headers = new Headers();
                    headers.append('Content-Type', 'application/json');
                    return headers;
                })(),
                method: 'POST',
                body: JSON.stringify('{-- replaced with object --}'),
            });
            const response = await fetch(request);
            if (!response.ok) {
                throw new Error(`HTTP error! Status: ${response.status}`);
            }
            return await response.text();
        } catch (error) {
            console.error('Fetch failed:', error);
            return '';
        }
    }
    function reScript(helper) {
        for (const node of helper.childNodes) {
            if (node.hasChildNodes()) {
                reScript(node);
            }
            if (node.nodeName === 'SCRIPT') {
                const script = document.createElement('script');
                script.type = 'text/javascript';
                script.textContent = node.textContent;
                node.replaceWith(script);
            }
        }
    }
    (async () => {
        const currentScript = document.currentScript;
        const content = await fetchShortcodeData();
        // console.log(content);
        const helper = document.createElement('div');
        helper.id = 'helper';
        helper.innerHTML = content;
        reScript(helper);
        currentScript.after(...helper.childNodes);
        currentScript.remove();
    })();
})();
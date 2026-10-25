// Dynamic Mermaid.js renderer for mdBook
(function () {
    function renderMermaid() {
        const blocks = document.querySelectorAll("pre code.language-mermaid");
        if (blocks.length === 0) return;

        blocks.forEach(function (code) {
            const pre = code.parentElement;
            const container = document.createElement("div");
            container.className = "mermaid";
            container.style.textAlign = "center";
            container.style.margin = "25px 0";
            container.textContent = code.textContent;
            pre.parentNode.replaceChild(container, pre);
        });

        const isDark = document.body.classList.contains("navy") ||
                       document.body.classList.contains("coal") ||
                       document.body.classList.contains("ayu");

        if (window.mermaid) {
            window.mermaid.initialize({
                startOnLoad: false,
                theme: isDark ? "dark" : "default",
                securityLevel: "loose"
            });
            window.mermaid.run();
        }
    }

    function loadMermaid() {
        if (window.mermaid) {
            renderMermaid();
            return;
        }
        const script = document.createElement("script");
        script.src = "https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js";
        script.onload = renderMermaid;
        document.head.appendChild(script);
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", loadMermaid);
    } else {
        loadMermaid();
    }
})();

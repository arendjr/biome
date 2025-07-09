---
"@biomejs/biome": patch
---

Improved performance of the scanner by optimising the handling of optionals (`T | undefined`), which are a very common occurrence.

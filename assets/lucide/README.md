# Lucide subset

39 official, unmodified SVGs from [lucide-icons/lucide](https://github.com/lucide-icons/lucide),
pinned at commit `20aef31db93fe4d9f398dbb15b120d99c48eef95` (`icons/*.svg`).
`github.svg` is the official [Feather v4.29.2](https://github.com/feathericons/feather/tree/v4.29.2/icons)
icon (MIT); that brand glyph is not in the pinned Lucide catalog.
40 icons, 14,295 bytes of SVG payload. No runtime network requests or additional GPUI engine.

`LICENSE` contains Lucide's ISC notice and the MIT notice for Feather-derived icons
and the GitHub glyph.
The component API and gallery layout are independently designed; no GPUI Kit code is included.

`IconName` is deliberately a small catalog. Add names and their upstream SVGs together;
applications can use `Icon::data` or `Icon::path` for everything else. The latter uses the
host's GPUI `AssetSource`; built-in names need no asset registration. SVGs are tinted as
monochrome masks by GPUI and inherit text color unless explicitly styled.

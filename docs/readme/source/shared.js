// Theme switch for rendered illustrations: ?theme=dark selects the dark tokens in the stylesheet.
document.documentElement.dataset.theme = new URLSearchParams(location.search).get("theme") === "dark" ? "dark" : "light";

// The IRYS eye, same geometry as src-vue/components/AnimatedEye.vue, plus a few glyphs, as one
// sprite so every figure can use <svg><use href="#eye"/></svg> and pick up the theme tokens.
document.addEventListener("DOMContentLoaded", () => {
  const eye = "M8 60c14-22 31-33 52-33s38 11 52 33c-14 22-31 33-52 33S22 82 8 60Z";
  document.body.insertAdjacentHTML("afterbegin", `
<svg width="0" height="0" style="position:absolute" aria-hidden="true">
  <defs>
    <radialGradient id="irisFill" cx="50%" cy="45%" r="60%">
      <stop offset="0%" style="stop-color:var(--iris-glow)"/>
      <stop offset="55%" style="stop-color:var(--iris)"/>
      <stop offset="100%" style="stop-color:var(--iris-deep)"/>
    </radialGradient>
    <linearGradient id="ringGrad" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" style="stop-color:var(--accent2)"/>
      <stop offset="100%" style="stop-color:var(--accent)"/>
    </linearGradient>
    <clipPath id="lidClip"><path d="${eye}"/></clipPath>
  </defs>
  <symbol id="eye" viewBox="0 0 120 120" overflow="visible">
    <g clip-path="url(#lidClip)">
      <path d="${eye}" style="fill:var(--sclera)"/>
      <circle cx="60" cy="60" r="19" fill="url(#irisFill)"/>
      <circle cx="60" cy="60" r="10" fill="#0c1418"/>
      <circle cx="53" cy="52" r="4" fill="#fff" opacity="0.85"/>
    </g>
    <path d="${eye}" fill="none" stroke="currentColor" stroke-width="3.5" stroke-linecap="round" opacity="0.95"/>
    <path d="M60 20v-9M92 29l6-7M28 29l-6-7" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" opacity="0.5"/>
  </symbol>
</svg>`);
});

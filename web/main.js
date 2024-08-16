import init, { render } from "./pkg/rustex.js";

const CANVAS_ID = "rustex";

async function run() {
  await init();
  render(CANVAS_ID);
  const canvas = document.getElementById(CANVAS_ID);
  const overlay = document.getElementById("coords");
  function updateCoordinates(event) {
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    overlay.textContent = `(${x}|${y})`;
  }

  canvas.addEventListener("mousemove", updateCoordinates);
}

run();

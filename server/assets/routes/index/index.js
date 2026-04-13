import init, { Counter } from "/assets/wasm/counter.js";

const counterRoot = document.querySelector("[data-ui='counter']");
const countElement = counterRoot.querySelector("[data-role='value']");
const decrementButton = counterRoot.querySelector("[data-action='decrement']");
const resetButton = counterRoot.querySelector("[data-action='reset']");
const incrementButton = counterRoot.querySelector("[data-action='increment']");

await init();

const counter = new Counter();

function renderCount() {
  countElement.textContent = counter.value();
}

function handleIncrement() {
  counter.increment();
  renderCount();
}

function handleDecrement() {
  counter.decrement();
  renderCount();
}

function handleReset() {
  counter.reset();
  renderCount();
}

decrementButton.addEventListener("click", handleDecrement);
resetButton.addEventListener("click", handleReset);
incrementButton.addEventListener("click", handleIncrement);

renderCount();

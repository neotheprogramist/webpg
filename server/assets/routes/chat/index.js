import init, { ChatClient } from "/assets/wasm/chat.js";

await init();

const chatRoot = document.querySelector("[data-ui='chat']");
const historyElement = chatRoot.querySelector("[data-role='history']");
const emptyStateElement = chatRoot.querySelector("[data-role='empty-state']");
const composerElement = chatRoot.querySelector("[data-role='composer']");
const inputElement = composerElement.querySelector("input[name='message']");

function appendMessage(message) {
  const item = document.createElement("li");
  item.className = "chat-message";
  item.dataset.ui = "chat-message";
  item.dataset.messageId = String(message.id);

  const body = document.createElement("div");
  body.className = "chat-message-body";
  body.textContent = message.body;

  const meta = document.createElement("div");
  meta.className = "chat-message-meta";
  meta.textContent = message.created_at;

  item.append(body, meta);
  historyElement.append(item);
  emptyStateElement.classList.add("is-hidden");
  historyElement.parentElement.scrollTop = historyElement.parentElement.scrollHeight;
}

function hexToBytes(hex) {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = Number.parseInt(hex.slice(i, i + 2), 16);
  }
  return bytes;
}

const fingerprint = chatRoot.dataset.certHash;
const transport = new WebTransport(
  new URL("/webtransport/chat", window.location.origin).toString(),
  {
    serverCertificateHashes: [
      { algorithm: "sha-256", value: hexToBytes(fingerprint) },
    ],
  },
);

await transport.ready;

const client = new ChatClient(transport, appendMessage);

composerElement.addEventListener("submit", async (event) => {
  event.preventDefault();
  const body = inputElement.value.trim();
  if (!body) return;
  await client.send(body);
  inputElement.value = "";
  inputElement.focus();
});

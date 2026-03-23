const modal = document.getElementById('input-modal');
const label = document.getElementById('modal-label');
const input = document.getElementById('modal-input');
const cancelBtn = document.getElementById('modal-cancel');
const okBtn = document.getElementById('modal-ok');

let currentResolve = null;

export function showInputModal(title, defaultValue = '') {
  return new Promise((resolve) => {
    currentResolve = resolve;
    label.textContent = title;
    input.value = defaultValue;
    modal.classList.add('open');
    input.focus();
    input.select();
  });
}

function close(value) {
  modal.classList.remove('open');
  if (currentResolve) {
    currentResolve(value);
    currentResolve = null;
  }
}

cancelBtn.onclick = () => close(null);
okBtn.onclick = () => close(input.value);

input.onkeydown = (e) => {
  if (e.key === 'Enter') {
    e.preventDefault();
    close(input.value);
  }
  if (e.key === 'Escape') {
    e.preventDefault();
    close(null);
  }
};

// Close on backdrop click
modal.onclick = (e) => {
  if (e.target === modal) close(null);
};

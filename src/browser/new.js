const newForm = document.getElementById('newGameForm')
const gameFld = document.getElementById('game')
const descriptionFld = document.getElementById('description')
const wordsFld = document.getElementById('words')
const submit = document.getElementById('make')

// See https://davidwalsh.name/javascript-debounce-
function debounce (func, delay) {
  let timer
  return function (...args) {
    clearTimeout(timer)
    timer = setTimeout(() => func.apply(this, args), delay)
  }
}

// Sanitize word list.
const sanitizeWords = () => {
  if (!wordsFld || wordsFld.value.length < 3) return
  let sanitized = wordsFld.value
  // Replace whitespace and punctuation with a single space.
  // Remove non-English alphabet characters.
  // Ensure each word is at least 3 and at most 10 letters long.
  sanitized = sanitized
    .replace(/[\s!"#$%&'()*+,-./:;<=>?@[\\\]^_`{|}~]/g, ' ')
    .replace(/[^a-zA-Z ]/g, '')
    .replace(/\b[a-zA-Z]{1,2}\b|\b[a-zA-Z]{11,}\b/g, '')
    .toUpperCase()
    .trim()
  // Ensure we have a maximum of 365 unique words.
  sanitized = [...new Set(sanitized.split(/\s+/))].slice(0,365).join(' ')
  wordsFld.value = sanitized
}

wordsFld.addEventListener('input', debounce(sanitizeWords, 5000))
wordsFld.addEventListener('change', sanitizeWords)

submit.addEventListener('click', async (e) => {
  e.preventDefault();
  const regStartResp = await fetch("/new", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      game: gameFld.value.trim(),
      description: descriptionFld.value.trim(),
      words: wordsFld.value,
    }),
  });
})

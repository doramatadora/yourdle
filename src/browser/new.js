const newForm = document.getElementById('newGameForm')
const inputs = ['game', 'description', 'words'].reduce((acc, input) => {
  acc[input] = {
    field: document.getElementById(input),
    validation: document.querySelector(`[data-validates="${input}"]`)
  }
  return acc
}, {})
const wordCount = document.getElementById('wordCount')
const submit = document.getElementById('make')

inputs.game.debounced = 3000
inputs.words.debounced = 8000

// See https://davidwalsh.name/javascript-debounce-
function debounce (func, delay) {
  let timer
  return function (...args) {
    clearTimeout(timer)
    timer = setTimeout(() => func.apply(this, args), delay)
  }
}

const validationMessage = ({ validation }, message) => {
  validation.innerText = message
  validation.style.display = 'block'
  return false
}

const hideValidation = ({ validation }) => {
  validation.style.display = 'none'
  return true
}

inputs.game.validate = async () => {
  if (inputs.game.field.value.trim().length < 3)
    return validationMessage(inputs.game, 'too short')
  const { ok } = await fetch('/validate', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      game: inputs.game.field.value.trim()
    })
  })
  return ok
    ? hideValidation(inputs.game)
    : validationMessage(inputs.game, 'game already exists')
}

inputs.description.validate = async () => {
  if (inputs.description.field.value.trim().length < 10)
    return validationMessage(inputs.description, 'too short')
  return hideValidation(inputs.description)
}

inputs.words.validate = () => {
  if (inputs.words.field.value.length < 3) {
    wordCount.innerText = ''
    return validationMessage(inputs.words, 'too short')
  }
  let sanitized = inputs.words.field.value
  // Replace whitespace and punctuation with a single space.
  // Remove non-ASCII alphabet characters.
  // Ensure each word is at least 3 and at most 10 letters long.
  sanitized = sanitized
    .replace(/[\s!"#$%&'()*+,-./:;<=>?@[\\\]^_`{|}~]/g, ' ')
    .replace(/[^a-zA-Z ]/g, '')
    .replace(/\b[a-zA-Z]{1,2}\b|\b[a-zA-Z]{11,}\b/g, '')
    .toUpperCase()
    .trim()
  // Ensure we have a maximum of 365 unique words.
  sanitized = [...new Set(sanitized.split(/\s+/))].slice(0, 365)
  if (sanitized.length >= 2) {
    inputs.words.field.value = sanitized.join(' ')
    wordCount.innerText = `[${sanitized.length}]`
    return sanitized.length < 7
      ? validationMessage(inputs.words, 'we need 7 words or more')
      : hideValidation(inputs.words)
  }
  return false
}

const processEventTargets = (listenerAction = 'addEventListener') => {
  Object.values(inputs).forEach(({ field, validate, debounced }) => {
    field[listenerAction](
      'input',
      debounced ? debounce(validate, debounced) : validate
    )
    field[listenerAction]('change', validate)
  })
}

processEventTargets()

submit.addEventListener('click', async e => {
  e.preventDefault()
  // Validate everything once more.
  const validationResults = await Promise.all(
    Object.values(inputs).map(async ({ validate }) => await validate())
  )
  if (validationResults.some(r => !r)) return

  processEventTargets('removeEventListener')

  const res = await fetch('/new', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      game: inputs.game.field.value.trim(),
      description: inputs.description.field.value.trim(),
      words: inputs.words.field.value.trim()
    })
  })
  if (!res.ok) return announce(`Something went wrong 🥲\nTry again later`)
  const slug = await res.text()
  newForm.remove()
  console.warn({ slug })
})

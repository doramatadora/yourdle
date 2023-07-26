const newForm = document.getElementById('newGameForm')
const createdSuccessfully = document.getElementById('createdSuccessfully')
const wordCount = document.getElementById('wordCount')
const submit = document.getElementById('make')
const gameLink = document.getElementById('gameLink')

const inputs = ['game', 'description', 'words'].reduce((acc, input) => {
  acc[input] = {
    field: document.getElementById(input),
    validation: document.querySelector(`[data-validates="${input}"]`)
  }
  return acc
}, {})

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
  Object.values(inputs).forEach(({ field, validate }) => {
    field[listenerAction]('change', validate)
  })
}

processEventTargets()

submit.addEventListener('click', async e => {
  e.preventDefault()
  processEventTargets('removeEventListener')
  // Validate everything and return early on failure.
  const validationResults = await Promise.all(
    Object.values(inputs).map(async ({ validate }) => await validate())
  )
  if (validationResults.some(r => !r)) return processEventTargets()

  const gameData = {
    game: inputs.game.field.value.trim(),
    description: inputs.description.field.value.trim(),
    words: inputs.words.field.value.trim()
  }
  // Attempt to create the game.
  const res = await fetch('/new', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(gameData)
  })
  if (!res.ok) {
    processEventTargets()
    return announce(`Something went wrong 🥲\nTry again later`)
  }
  const gameSlug = await res.text()
  const gameUrl = `${window.location.origin}/${gameSlug}`
  gameLink.href = gameUrl
  gameLink.innerText = `${window.location.host}/${gameSlug}`
  clipboard.value = [
    `I made a word game! Check out "${gameData.game}" at:`,
    gameUrl
  ].join('\n')
  newForm.remove()
  createdSuccessfully.style.display = 'block'
})

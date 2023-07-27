const TRIES = 6
const colorMode = document.getElementById('colorMode')
const share = document.getElementById('share')
const shareThis = document.getElementById('shareThis')
const gameTitle = document.getElementById('gameTitle')
const announcer = document.getElementById('announcer')
const showInfo = document.getElementById('showInfo')
const info = document.getElementById('info')
const showStats = document.getElementById('showStats')
const stats = document.getElementById('stats')
const showFeedback = document.getElementById('showFeedback')
const feedback = document.getElementById('feedback')
const feedbackText = document.getElementById('feedbackText')
const sendFeedback = document.getElementById('sendFeedback')
const body = document.querySelector('body')
const activeRow = document.getElementsByClassName('active')
const clipboard = document.getElementById('clipboard')
const buttons = {}

const [, gameSlug] = window.location.pathname.split('/')
const origin = window.location.origin

if (localStorage.getItem('hiContrast') === 'true')
  body.classList.add('hiContrast')

const opn = el => {
  closeAll()
  el.style.display = 'block'
}

const closeAll = () => {
  ;[info, stats, announcer, feedback].forEach(el =>
    el ? (el.style.display = 'none') : () => {}
  )
}

const announce = msg => {
  announcer.innerText = msg
  opn(announcer)
  setTimeout(() => {
    announcer.style.display = 'none'
  }, 3000)
}

const recordResult = stats => {
  const [guess, outcome] = stats.outcome[stats.outcome.length - 1]
  if (!activeRow.length) return
  let win = true
  // Update tiles with colours.
  const activeTiles = activeRow.item(0).children
  outcome.forEach((o, idx) => {
    const tile = activeTiles.item(idx)
    const currState = tile.dataset.state
    buttons[guess[idx].toLowerCase()].dataset.state =
      currState && o < currState ? currState : o
    tile.classList.add('flip')
    tile.dataset.state = o
    if (o !== 'correct') win = false
  })
  if (win) {
    announce('You win ❤️')
    setTimeout(() => {
      updateStats(stats)
    }, 2000)
  } else if (activeRow.item(0).nextElementSibling) {
    activeRow.item(0).nextElementSibling.classList.add('active')
  } else {
    announce('Better luck next time 😓')
  }
  activeRow.item(0).classList.remove('active')
}

const updateStats = state => {
  const intPerc = (num, total) => (total ? parseInt((num / total) * 100) : 0)
  const [games, winRate, streak, maxStreak] = stats.querySelectorAll('.stat>h4')
  const distro = stats.querySelectorAll('.dist>.bar')
  if (state && state.games) {
    games.innerText = state.games
    winRate.innerText =
      intPerc(
        state.distribution.reduce((a, v) => a + v, 0),
        state.games
      ) + '%'
    streak.innerText = state.streak
    maxStreak.innerText = state.maxStreak
    distro.forEach((bar, idx) => {
      bar.children[0].innerText = state.distribution[idx]
    })
    if (state.today === state.lastWin) {
      const c = {
        correct: '🟢',
        near: '🟡',
        wrong: '⚫'
      }
      const text = [
        `I took ${state.outcome.length} ${
          state.outcome.length === 1 ? `guess` : `guesses`
        } at today's ${gameTitle.value}:`
      ]
      state.outcome.forEach(o => {
        text.push(o[1].map(outcome => c[outcome]).join(' '))
      })
      text.push(`${origin}/${gameSlug}`)
      clipboard.value = text.join('\n')
      share.style.display = 'block'
    } else share.style.display = 'none'
  }
  const totalGames = parseInt(games.innerText) || 0
  if (totalGames) {
    distro.forEach(bar => {
      bar.style.height =
        30 + intPerc(bar.children[0].innerText, totalGames) + 'px'
    })
  }
  opn(stats)
}

const submitFeedback = () => {
  const fb = feedbackText.value.trim()
  if (fb.length < 10 || fb.length > 140) return
  const clearAndClose = msg => {
    feedbackText.value = ''
    closeAll()
    announce(msg)
  }
  fetch('/feedback', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      path: gameSlug,
      feedback: fb
    })
  })
    .then(res => {
      if (!res.ok) throw new Error(res.status)
      clearAndClose(`Thank you!`)
    })
    .catch(() => clearAndClose(`Something went wrong 🥲\nTry again later`))
}

const fallbackCopyTextToClipboard = text => {
  clipboard.value = text
  clipboard.focus()
  clipboard.select()
  let copied = false
  try {
    copied = document.execCommand('copy')
  } catch (err) {
    copied = false
  }
  return copied
}

const copyTextToClipboard = async text =>
  navigator.clipboard
    ? navigator.clipboard
        .writeText(text)
        .then(() => true)
        .catch(() => false)
    : fallbackCopyTextToClipboard(text)

const doClipboard = text =>
  copyTextToClipboard(text).then(ok => {
    announce(ok ? `Copied to clipboard\n📋` : `Copy:\n\n${text}`)
  })

document.addEventListener('keydown', e => {
  const key = e.key.toLowerCase()
  if (key === 'escape' || key === 'esc') closeAll()
  else if (!e.target.closest('#info,#stats,#feedback') && buttons[key])
    buttons[key].click()
})

document.addEventListener(
  'click',
  e => {
    if (!e.target.hasAttribute('target')) {
      switch (e.target) {
        case share:
          e.preventDefault()
          doClipboard(clipboard.value)
          break
        case shareThis:
          e.preventDefault()
          doClipboard(
            `Check out this fun word game, ${gameTitle.value}: ${origin}/${gameSlug}`
          )
          break
        case colorMode:
          e.preventDefault()
          e.target.blur()
          body.classList.toggle('hiContrast')
          localStorage.setItem(
            'hiContrast',
            body.classList.contains('hiContrast')
          )
          break
        case showInfo:
          e.preventDefault()
          opn(info)
          break
        case showStats:
          e.preventDefault()
          updateStats()
          break
        case showFeedback:
          e.preventDefault()
          opn(feedback)
          break
        case sendFeedback:
          e.preventDefault()
          submitFeedback()
          break
        default:
          if (
            e.target.classList.contains('close') ||
            !e.target.closest('#info,#stats,#feedback')
          ) {
            e.preventDefault()
            closeAll()
          }
          break
      }
    }
  },
  false
)

if (!['new', 'feedback', 'validate'].includes(gameSlug)) {
  for (const btn of document.querySelectorAll('button[data-key]').values()) {
    buttons[btn.dataset.key] = btn
    btn.addEventListener('click', e => {
      e.stopPropagation()
      closeAll()
      if (!activeRow.length) return
      const tiles = Array.from(activeRow.item(0).children)
      const lastBlankIndex = tiles.findIndex(t => !t.innerText.length)
      switch (e.target.dataset.key) {
        case 'enter':
          if (lastBlankIndex != -1) return
          const word = tiles
            .map(t => {
              t.classList.remove('pulse')
              return t.innerText
            })
            .join('')
          fetch(`${gameSlug}?guess=${word}`, {
            method: 'GET',
            credentials: 'same-origin'
          })
            .then(res => {
              if (res.ok) return res.json()
              throw new Error(res.status)
            })
            .then(res => recordResult(res))
            .catch(({ message }) =>
              announce(
                message === '404'
                  ? `That's not on the list`
                  : `Something went wrong`
              )
            )
          break
        case 'backspace':
          const toExpunge =
            tiles[lastBlankIndex <= 0 ? tiles.length - 1 : lastBlankIndex - 1]
          toExpunge.innerText = ''
          toExpunge.classList.remove('pulse')
          break
        default:
          if (lastBlankIndex === -1) return
          tiles[lastBlankIndex].classList.add('pulse')
          tiles[lastBlankIndex].innerText = e.target.dataset.key.toUpperCase()
          break
      }
    })
  }
  if (stats && stats.getAttribute('data-won-today') === 'true') {
    updateStats()
  }
}

/**
 * Dice roller plugin — /roll NdM
 * Examples: /roll 2d6, /roll 1d20, /roll 3d8
 */
export default {
  name: 'dice',
  version: '1.0.0',
  commands: [
    {
      name: 'roll',
      description: 'Roll dice',
      usage: '/roll NdM (e.g. /roll 2d6)',
      handler(args, { sendMessage }) {
        const match = args.match(/^(\d+)d(\d+)$/i)
        if (!match) {
          sendMessage('Usage: /roll NdM (e.g. /roll 2d6)')
          return
        }

        const count = Math.min(parseInt(match[1], 10), 100)
        const sides = Math.min(parseInt(match[2], 10), 1000)

        if (count < 1 || sides < 1) {
          sendMessage('Invalid dice: count and sides must be at least 1')
          return
        }

        const rolls = []
        for (let i = 0; i < count; i++) {
          rolls.push(Math.floor(Math.random() * sides) + 1)
        }

        const total = rolls.reduce((a, b) => a + b, 0)
        const detail = count > 1 ? ` [${rolls.join(', ')}]` : ''
        sendMessage(`🎲 Rolled ${count}d${sides}: **${total}**${detail}`)
      },
    },
  ],
}

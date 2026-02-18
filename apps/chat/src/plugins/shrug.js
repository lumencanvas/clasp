/**
 * Shrug plugin — /shrug [optional text]
 * Appends the shrug kaomoji to your message.
 */
export default {
  name: 'shrug',
  version: '1.0.0',
  commands: [
    {
      name: 'shrug',
      description: 'Append a shrug',
      usage: '/shrug [message]',
      handler(args, { sendMessage }) {
        const text = args ? `${args} ¯\\_(ツ)_/¯` : '¯\\_(ツ)_/¯'
        sendMessage(text)
      },
    },
  ],
}

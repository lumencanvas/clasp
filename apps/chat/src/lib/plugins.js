/**
 * Plugin system for CLASP Chat.
 *
 * Plugin interface:
 *   { name, version, commands, init(api), destroy() }
 *
 * ChatAPI provided to plugins:
 *   { onMessage, sendMessage, registerCommand, getCurrentRoom, getUser }
 */

const plugins = new Map()
const commands = new Map() // name -> { description, usage, handler, pluginName }
const messageHandlers = []

/**
 * Register a plugin.
 */
export function registerPlugin(plugin, api) {
  if (plugins.has(plugin.name)) return

  const pluginApi = {
    ...api,
    registerCommand(name, { description, usage, handler }) {
      commands.set(name, { description, usage, handler, pluginName: plugin.name })
    },
  }

  plugins.set(plugin.name, plugin)

  // Auto-register commands from plugin definition
  if (plugin.commands) {
    for (const cmd of plugin.commands) {
      commands.set(cmd.name, {
        description: cmd.description,
        usage: cmd.usage,
        handler: cmd.handler,
        pluginName: plugin.name,
      })
    }
  }

  if (plugin.init) {
    plugin.init(pluginApi)
  }
}

/**
 * Unregister a plugin and remove its commands.
 */
export function unregisterPlugin(name) {
  const plugin = plugins.get(name)
  if (!plugin) return

  // Remove commands owned by this plugin
  for (const [cmdName, cmd] of commands) {
    if (cmd.pluginName === name) {
      commands.delete(cmdName)
    }
  }

  if (plugin.destroy) plugin.destroy()
  plugins.delete(name)
}

/**
 * Execute a slash command. Returns true if handled.
 */
export function executeCommand(commandText, context) {
  const match = commandText.match(/^\/(\S+)\s*(.*)$/)
  if (!match) return false

  const [, name, args] = match
  const cmd = commands.get(name)
  if (!cmd) return false

  cmd.handler(args.trim(), context)
  return true
}

/**
 * Get all registered commands for autocomplete.
 */
export function getRegisteredCommands() {
  return Array.from(commands.entries()).map(([name, cmd]) => ({
    name,
    description: cmd.description,
    usage: cmd.usage,
  }))
}

/**
 * Initialize built-in plugins.
 */
export async function initBuiltinPlugins(api) {
  // Import and register built-in plugins
  const [dice, shrug] = await Promise.all([
    import('../plugins/dice.js'),
    import('../plugins/shrug.js'),
  ])
  registerPlugin(dice.default, api)
  registerPlugin(shrug.default, api)
}

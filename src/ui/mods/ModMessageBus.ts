/**
 * ModMessageBus — Singleton for routing messages between Kokoro host and MOD iframes.
 *
 * Tracks registered iframe windows by component name and provides
 * targeted send / broadcast capabilities.
 */

export interface ModBusMessage {
    type: "prop-update" | "event";
    payload?: unknown;
}

class ModMessageBus {
    /** Map of component slot name → { window, origin } */
    private windows = new Map<string, { win: Window; origin: string }>();

    /** Register an iframe's contentWindow for a given component name. */
    register(name: string, win: Window, origin = '*') {
        this.windows.set(name, { win, origin });
        console.log(`[ModMessageBus] Registered component '${name}' (origin: ${origin})`);
    }

    /** Unregister a component by name. */
    unregister(name: string) {
        this.windows.delete(name);
        console.log(`[ModMessageBus] Unregistered component '${name}'`);
    }

    /** Send a message to a specific component iframe. */
    send(name: string, message: ModBusMessage) {
        const entry = this.windows.get(name);
        if (entry) {
            entry.win.postMessage(message, entry.origin);
        } else {
            console.warn(`[ModMessageBus] No iframe registered for component '${name}'`);
        }
    }

    /** Broadcast a message to all registered mod iframes. */
    broadcast(message: ModBusMessage) {
        for (const { win, origin } of this.windows.values()) {
            win.postMessage(message, origin);
        }
    }

    /** Check if a component is registered. */
    has(name: string): boolean {
        return this.windows.has(name);
    }
}

export const modMessageBus = new ModMessageBus();

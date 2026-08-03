/**
 * Reactive view of the Rust clock.
 *
 * This composable deliberately runs **no timer of its own**. It seeds from
 * `get_snapshot` and then only ever reflects what Rust pushes, which is what
 * keeps the UI honest when the window has been hidden or throttled.
 */

import { listen } from '@tauri-apps/api/event'
import { onScopeDispose, ref, shallowRef } from 'vue'

import { getSnapshot } from '@/lib/ipc'
import { EVENT_TICK, type Snapshot } from '@/lib/types'

export function useTimer() {
  const snapshot = shallowRef<Snapshot | null>(null)
  const ready = ref(false)

  // Seed immediately so the first paint is not blank while we wait up to a
  // second for the next tick.
  void getSnapshot()
    .then((initial) => {
      snapshot.value ??= initial
      ready.value = true
    })
    .catch(() => {
      // No snapshot yet is survivable — the next tick fixes it.
      ready.value = true
    })

  const stop = listen<Snapshot>(EVENT_TICK, (event) => {
    snapshot.value = event.payload
    ready.value = true
  })

  onScopeDispose(() => {
    void stop.then((unlisten) => unlisten())
  })

  return { snapshot, ready }
}

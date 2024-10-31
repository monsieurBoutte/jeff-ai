import { setup } from 'xstate';

export type ModeEvent =
  | { type: 'TOGGLE_REWRITE' }
  | { type: 'TOGGLE_RECORDER' };

export const modeMachine = setup({
  types: {
    events: {} as ModeEvent
  }
}).createMachine({
  id: 'mode',
  initial: 'idle',
  states: {
    idle: {
      on: {
        TOGGLE_REWRITE: {
          target: 'rewrite'
        },
        TOGGLE_RECORDER: {
          target: 'recorder'
        }
      }
    },
    rewrite: {
      on: {
        TOGGLE_REWRITE: {
          target: 'idle'
        },
        TOGGLE_RECORDER: {
          target: 'recorder'
        }
      }
    },
    recorder: {
      on: {
        TOGGLE_REWRITE: {
          target: 'rewrite'
        },
        TOGGLE_RECORDER: {
          target: 'idle'
        }
      }
    }
  }
});

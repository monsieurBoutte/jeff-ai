import { createMachine } from 'xstate';

export const modeMachine = createMachine({
  id: 'mode',
  initial: 'idle',
  states: {
    idle: {
      on: {
        TOGGLE_RECORDER: 'recorder',
        TOGGLE_SYSTEM_OUTPUT: 'systemOutput'
      }
    },
    recorder: {
      on: {
        TOGGLE_RECORDER: 'idle',
        TOGGLE_SYSTEM_OUTPUT: 'systemOutput'
      }
    },
    systemOutput: {
      on: {
        TOGGLE_SYSTEM_OUTPUT: 'idle',
        TOGGLE_RECORDER: 'recorder'
      }
    }
  }
});

import { createMachine, assign } from 'xstate';

type DeleteEvent =
  | { type: 'START_DELETE'; taskId: string }
  | { type: 'PAUSE' }
  | { type: 'RESUME' }
  | { type: 'CANCEL' }
  | { type: 'CONFIRM_DELETE' };

interface DeleteContext {
  taskId: string | null;
}

export const deleteTaskMachine = createMachine({
  types: {} as {
    context: DeleteContext;
    events: DeleteEvent;
  },
  id: 'deleteTask',
  initial: 'idle',
  context: {
    taskId: null
  },
  states: {
    idle: {
      on: {
        START_DELETE: {
          target: 'deleting',
          actions: assign({
            taskId: (_, event) => event.taskId
          })
        }
      }
    },
    deleting: {
      on: {
        PAUSE: 'paused',
        CANCEL: 'idle',
        CONFIRM_DELETE: 'idle'
      },
      after: {
        2000: 'idle'
      }
    },
    paused: {
      on: {
        RESUME: 'deleting',
        CANCEL: 'idle',
        CONFIRM_DELETE: 'idle'
      }
    }
  }
});

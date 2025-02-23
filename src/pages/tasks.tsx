import { useCallback, useEffect, useState, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '@/lib/utils';
import dayjs from 'dayjs';
import { AnimatePresence } from 'motion/react';
import { TaskEditor } from '@/components/task-editor';
import useSound from 'use-sound';
import recordSfx from '@/assets/cassette_tape_record.mp3';
import { listen } from '@tauri-apps/api/event';
import { MicrophoneToggle } from '@/components/microphone-toggle';
import { Button } from '@/components/ui/button';
import * as motion from 'motion/react-client';
import { useUserSettings } from '@/hooks/use-user-settings';
import { WeatherResponse } from '@/types/commands';
import {
  Task,
  ApiTask,
  DaySection,
  TranscriptionEvent,
  transformApiTask
} from '@/components/tasks/types';
import { WeekSection } from '@/components/tasks/WeekSection';
import { TaskList } from '@/components/tasks/TaskList';

export default function Tasks() {
  const {
    isAuthenticated,
    getToken,
    settings: existingSettings
  } = useUserSettings();

  const [selectedDay, setSelectedDay] = useState<string>('MONDAY');
  const [tasks, setTasks] = useState<Task[]>([]);
  const [play] = useSound(recordSfx);
  const [isRecording, setIsRecording] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [deletingTaskId, setDeletingTaskId] = useState<number | null>(null);
  const deleteTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const longPressTimeout = useRef<NodeJS.Timeout | null>(null);
  const [isHoveringDeleteArea, setIsHoveringDeleteArea] = useState(false);
  const [weekSections, setWeekSections] = useState<DaySection[]>([]);

  const fetchWeatherData = useCallback(
    async (days: DaySection[]) => {
      if (
        !isAuthenticated ||
        !getToken ||
        !existingSettings?.lat ||
        !existingSettings?.lon
      ) {
        return days;
      }

      try {
        const token = await getToken();
        const weatherData = await invoke<WeatherResponse>(
          'get_weather_forecast',
          {
            token,
            lat: existingSettings.lat,
            lon: existingSettings.lon
          }
        );

        if (weatherData?.data?.daily?.length > 0) {
          return days.map((day) => {
            const dayDate = dayjs(day.date).startOf('day');
            const weatherDay = weatherData.data.daily.find((weatherDay) => {
              const weatherDate = dayjs.unix(weatherDay.dt).startOf('day');
              return weatherDate.isSame(dayDate, 'day');
            });

            if (weatherDay?.temp) {
              return {
                ...day,
                temperature: `${Math.round(
                  weatherDay.temp.min
                )}° - ${Math.round(weatherDay.temp.max)}°`
              };
            }
            return day;
          });
        }
      } catch (error) {
        console.error('Error fetching weather data:', error);
      }
      return days;
    },
    [isAuthenticated, getToken, existingSettings]
  );

  useEffect(() => {
    const generateWeekSections = async () => {
      const today = dayjs();
      const monday = today.startOf('week').add(1, 'day');

      const days = ['MONDAY', 'TUESDAY', 'WEDNESDAY', 'THURSDAY', 'FRIDAY'].map(
        (day, index) => ({
          day,
          date: monday.add(index, 'day').format('MMMM D, YYYY'),
          temperature: ''
        })
      );

      const sectionsWithWeather = await fetchWeatherData(days);
      setWeekSections(sectionsWithWeather);
    };

    generateWeekSections();
  }, [fetchWeatherData]);

  const fetchTasks = useCallback(async () => {
    if (!isAuthenticated || !getToken) return;

    try {
      const token = await getToken();
      const response = await invoke<ApiTask[]>('fetch_tasks', { token });
      if (Array.isArray(response)) {
        const transformedTasks = response.map(transformApiTask);
        setTasks(transformedTasks);
      }
    } catch (error) {
      console.error('Error fetching tasks:', error);
    }
  }, [getToken, isAuthenticated]);

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  useEffect(() => {
    const unlisten = listen(
      'transcription-complete',
      (event: TranscriptionEvent) => {
        const selectedSection = weekSections.find((s) => s.day === selectedDay);
        if (!selectedSection || !selectedSection.date) return;

        const newTask: Task = {
          id: Date.now(),
          content: event.payload,
          completed: false,
          assignedDate: dayjs(selectedSection.date).toISOString()
        };
        setTasks((prev) => [...prev, newTask]);
      }
    );

    return () => {
      unlisten.then((unlistenFn) => unlistenFn());
    };
  }, [selectedDay, weekSections]);

  const handleRecording = async () => {
    if (!isAuthenticated || !getToken) return;
    const token = await getToken();

    if (!isRecording) {
      play();
      await invoke('start_recording');
      setIsRecording(true);
    } else {
      play();
      setIsRecording(false);
      setIsProcessing(true);
      try {
        await invoke('stop_recording', { token, refine: true });
      } finally {
        setIsProcessing(false);
      }
    }
  };

  const handleAddTask = async (content: string, isNewTask?: boolean) => {
    if (!content.trim() || !isAuthenticated || !getToken) return;

    const tempId = Date.now();
    const optimisticTask: Task = {
      id: tempId,
      content,
      completed: false,
      assignedDate: dayjs(
        weekSections.find((s) => s.day === selectedDay)?.date
      ).toISOString(),
      __tempId: tempId,
      isNewTask
    };

    setTasks((prev) => [...prev, optimisticTask]);

    try {
      const token = await getToken();
      const response = await invoke<ApiTask>('create_task', {
        token,
        content,
        assignedDate: optimisticTask.assignedDate
      });

      setTasks((prev) =>
        prev.map((task) =>
          task.__tempId === tempId ? transformApiTask(response) : task
        )
      );
    } catch (error) {
      console.error('Error creating task:', error);
      setTasks((prev) => prev.filter((task) => task.__tempId !== tempId));
    }
  };

  const handleToggleTask = async (id: number) => {
    const task = tasks.find((t) => t.id === id);
    if (!task || !isAuthenticated || !getToken) return;

    setTasks((prev) =>
      prev.map((task) =>
        task.id === id ? { ...task, completed: !task.completed } : task
      )
    );

    try {
      const token = await getToken();
      await invoke('update_task', {
        token,
        taskId: id,
        completed: !task.completed
      });
    } catch (error) {
      console.error('Error updating task:', error);
      setTasks((prev) =>
        prev.map((task) =>
          task.id === id ? { ...task, completed: task.completed } : task
        )
      );
    }
  };

  const handleDeleteTask = async (taskId: number) => {
    if (!isAuthenticated || !getToken) return;

    setTasks((prev) => prev.filter((task) => task.id !== taskId));

    try {
      const token = await getToken();
      await invoke('delete_task', { token, taskId });
    } catch (error) {
      console.error('Error deleting task:', error);
      fetchTasks();
    } finally {
      setDeletingTaskId(null);
    }
  };

  const handleLongPressStart = (taskId: number) => {
    longPressTimeout.current = setTimeout(() => {
      setDeletingTaskId(taskId);
    }, 500);
  };

  const handleLongPressEnd = () => {
    if (longPressTimeout.current) {
      clearTimeout(longPressTimeout.current);
      longPressTimeout.current = null;
    }
  };

  useEffect(() => {
    if (deletingTaskId && !isHoveringDeleteArea) {
      const timer = setTimeout(() => {
        setDeletingTaskId(null);
      }, 2000);
      deleteTimeoutRef.current = timer;

      return () => {
        if (deleteTimeoutRef.current) {
          clearTimeout(deleteTimeoutRef.current);
        }
      };
    }
  }, [deletingTaskId, isHoveringDeleteArea]);

  const tasksForSelectedDay = tasks.filter((t) =>
    dayjs(t.assignedDate).isSame(
      weekSections.find((s) => s.day === selectedDay)?.date,
      'day'
    )
  );

  return (
    <div className="max-w-2xl mx-auto">
      <div className="space-y-0">
        {weekSections.map((section, index) => (
          <WeekSection
            key={section.day}
            section={section}
            index={index}
            selectedDay={selectedDay}
            isRecording={isRecording}
            isProcessing={isProcessing}
            onDaySelect={setSelectedDay}
            onRecording={handleRecording}
            onAddTask={handleAddTask}
          >
            <TaskList
              tasks={tasksForSelectedDay}
              deletingTaskId={deletingTaskId}
              onToggleTask={handleToggleTask}
              onDeleteTask={handleDeleteTask}
              onLongPressStart={handleLongPressStart}
              onLongPressEnd={handleLongPressEnd}
              onHoverDeleteArea={setIsHoveringDeleteArea}
            />
          </WeekSection>
        ))}
      </div>
    </div>
  );
}

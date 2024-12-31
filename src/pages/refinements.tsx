import { useCallback, useEffect, useState, useMemo } from 'react';
import { useKindeAuth } from '@kinde-oss/kinde-auth-react';
import { invoke } from '@tauri-apps/api/core';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { Refinement } from '@/types/commands';
import dayjs from 'dayjs';
import { Copy } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useToast } from '@/hooks/use-toast';

export default function Refinements() {
  const { isAuthenticated, getToken } = useKindeAuth();
  const [refinements, setRefinements] = useState<Refinement[]>([]);

  const { toast } = useToast();

  const handleGetAllRefinements = useCallback(async () => {
    if (!isAuthenticated || !getToken) {
      return;
    }

    const token = await getToken();
    const fetchedRefinements = await invoke<Array<Refinement>>(
      'get_all_refinements',
      {
        token
      }
    );
    setRefinements(fetchedRefinements);
  }, [isAuthenticated, getToken]);

  useEffect(() => {
    handleGetAllRefinements();
  }, [handleGetAllRefinements]);

  const groupedRefinements = useMemo(() => {
    // First sort all refinements by creation time (newest first)
    const sortedRefinements = [...refinements].sort((a, b) =>
      dayjs(b.createdAt).diff(dayjs(a.createdAt))
    );

    // Then group the sorted refinements by date
    return sortedRefinements.reduce((acc, refinement) => {
      const date = dayjs(refinement.createdAt).format('YYYY-MM-DD');
      if (!acc[date]) {
        acc[date] = [];
      }
      acc[date].push(refinement);
      return acc;
    }, {} as Record<string, Refinement[]>);
  }, [refinements]);

  const sortedDates = useMemo(() => {
    return Object.keys(groupedRefinements).sort((a, b) =>
      dayjs(b).diff(dayjs(a))
    );
  }, [groupedRefinements]);

  const formatDate = (date: string) => {
    const today = dayjs().startOf('day');
    const yesterday = dayjs().subtract(1, 'day').startOf('day');
    const refinementDate = dayjs(date);

    if (refinementDate.isSame(today, 'day')) {
      return 'Today';
    } else if (refinementDate.isSame(yesterday, 'day')) {
      return 'Yesterday';
    } else {
      return refinementDate.format('MMMM D, YYYY');
    }
  };

  return (
    <section>
      <div className="max-w-3xl mx-auto">
        {sortedDates.map((date) => (
          <div key={date} className="mb-8">
            <h2 className="text-2xl font-semibold text-gray-700 dark:text-gray-300 mb-4">
              {formatDate(date)}
            </h2>
            <div className="space-y-4">
              {groupedRefinements[date].map((refinement) => (
                <div
                  key={refinement.id}
                  className="p-6 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 shadow-sm hover:shadow-md transition-shadow duration-200"
                >
                  <div className="flex justify-between items-start mb-6">
                    <time className="text-xs text-gray-500 dark:text-gray-400">
                      {dayjs(refinement.createdAt).format('MM/DD/YY h:mm A')}
                    </time>
                  </div>

                  <div className="grid gap-6">
                    <div>
                      <div className="flex gap-1 items-center mb-3 -ml-[3px]">
                        <span className="w-2 h-2 bg-gray-600 rounded-full mr-2" />
                        <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                          Original Text
                        </h3>
                      </div>
                      <div className="pl-4 border-l-2 border-gray-100 dark:border-gray-700">
                        <p className="text-gray-700 dark:text-gray-300">
                          {refinement.originalText}
                        </p>
                        <div className="flex items-center gap-2 mt-1">
                          <span className="text-xs text-gray-500 dark:text-gray-400 block">
                            {refinement.originalTextWordCount} words
                          </span>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="size-6"
                            onClick={() => {
                              writeText(refinement.originalText);
                              toast({
                                description: 'original text copied to clipboard'
                              });
                            }}
                          >
                            <Copy className="h-[0.9rem] w-[0.9rem]" />
                            <span className="sr-only">Copy originaltext</span>
                          </Button>
                        </div>
                      </div>
                    </div>

                    <div>
                      <div className="flex gap-1 items-center mb-3 -ml-[3px]">
                        <span className="w-2 h-2 bg-indigo-500 rounded-full mr-2" />
                        <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                          Refined Text
                        </h3>
                      </div>
                      <div className="pl-4 border-l-2 border-gray-100 dark:border-gray-700">
                        <p className="text-gray-700 dark:text-gray-300">
                          {refinement.refinedText}
                        </p>
                        <div className="flex items-center gap-2 mt-1">
                          <span className="text-xs text-gray-500 dark:text-gray-400 block">
                            {refinement.refinedTextWordCount} words
                          </span>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="size-6"
                            onClick={() => {
                              writeText(refinement.refinedText);
                              toast({
                                description: 'refined text copied to clipboard'
                              });
                            }}
                          >
                            <Copy className="h-[0.9rem] w-[0.9rem]" />
                            <span className="sr-only">Copy refined text</span>
                          </Button>
                        </div>
                      </div>
                    </div>

                    {refinement.explanation && (
                      <div>
                        <div className="flex gap-1 items-center mb-3 -ml-[3px]">
                          <span className="w-2 h-2 bg-gray-400 dark:bg-gray-500 rounded-full mr-2" />
                          <h3 className="text-sm font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider">
                            Explanation
                          </h3>
                        </div>
                        <div className="pl-4 border-l-2 border-gray-100 dark:border-gray-700">
                          <p className="text-gray-400 dark:text-gray-500">
                            {refinement.explanation}
                          </p>
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

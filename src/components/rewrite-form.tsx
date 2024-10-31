import { useCallback, useState } from 'react';
import { useForm } from 'react-hook-form';
import { ClipboardCopy } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Form, FormControl, FormField, FormItem } from '@/components/ui/form';
import { useToast } from '@/hooks/use-toast';
import type { RefineMessageResponse } from '@/types/commands';

interface RewriteFormValues {
  messageToRewrite: string;
}

export const RewriteForm = () => {
  const [refinedMsg, setRefinedMsg] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  const { toast } = useToast();

  const form = useForm<RewriteFormValues>({
    defaultValues: {
      messageToRewrite: ''
    }
  });

  const handleRewriteSubmission = useCallback(
    async (values: RewriteFormValues) => {
      setIsLoading(true);
      const response = await invoke<RefineMessageResponse>('refine_message', {
        msg: values.messageToRewrite
      });
      setRefinedMsg(response.suggested_message_rewrite);
      setIsLoading(false);
      toast({
        description: 'Refined message copied to clipboard'
      });
    },
    []
  );

  const handleCopyToClipboard = useCallback(() => {
    if (refinedMsg) {
      navigator.clipboard.writeText(refinedMsg);
      toast({
        description: 'Refined message copied to clipboard'
      });
    }
  }, [refinedMsg, toast]);

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      // Submit form on Cmd+Enter (Mac) or Ctrl+Enter (Windows/Linux)
      if (
        event.key === 'Enter' &&
        (event.metaKey || event.ctrlKey) // metaKey is Cmd on Mac, ctrlKey is Ctrl on Windows/Linux
      ) {
        event.preventDefault();
        form.handleSubmit(handleRewriteSubmission)();
      }
    },
    [form, handleRewriteSubmission]
  );

  return (
    <Form {...form}>
      <form
        onSubmit={form.handleSubmit(handleRewriteSubmission)}
        onKeyDown={handleKeyDown}
        className="flex flex-col gap-2"
      >
        <Button
          type="submit"
          variant="outline"
          disabled={!form.watch('messageToRewrite') || isLoading}
        >
          {isLoading ? 'Refining...' : 'Refine'}
        </Button>
        <FormField
          control={form.control}
          name="messageToRewrite"
          render={({ field }) => (
            <FormItem>
              <FormControl>
                <Textarea placeholder="...enter message to refine" {...field} />
              </FormControl>
            </FormItem>
          )}
        />
        {refinedMsg && (
          <div className="mt-2 flex flex-col gap-2">
            <div className="flex justify-between items-center">
              <h2 className="text-lg font-semibold">Refined message</h2>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                onClick={handleCopyToClipboard}
                className="h-8 w-8"
              >
                <ClipboardCopy className="h-4 w-4" />
                <span className="sr-only">
                  Copy refined message to clipboard
                </span>
              </Button>
            </div>
            <p className="pl-2 text-sm border-l-2 border-gray-800 dark:border-white">
              {refinedMsg}
            </p>
          </div>
        )}
      </form>
    </Form>
  );
};

import { Voicemail } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface MicrophoneToggleProps {
  isActive: boolean;
  onClick: () => void;
}

export function MicrophoneToggle({ isActive, onClick }: MicrophoneToggleProps) {
  return (
    <Button
      variant="outline"
      size="icon"
      onClick={onClick}
      className={cn(
        'transition-colors',
        isActive &&
          'border-red-500 animate-recording-pulse bg-red-50 dark:hover:bg-red-950'
      )}
    >
      <Voicemail
        className="h-[1.2rem] w-[1.2rem]"
        color={isActive ? 'red' : 'currentColor'}
      />
      <span className="sr-only">Toggle microphone</span>
    </Button>
  );
}

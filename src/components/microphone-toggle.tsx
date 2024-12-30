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
      size="sm"
      onClick={onClick}
      className={cn(
        'transition-colors',
        isActive &&
          'border-red-500 text-red-800 animate-recording-pulse bg-red-100 dark:bg-red-950'
      )}
    >
      <Voicemail className="h-[1.2rem] w-[1.2rem]" />
      <span>{isActive ? 'Stop recording' : 'Use Speech to Text'}</span>
    </Button>
  );
}

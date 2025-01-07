import { Speaker } from 'lucide-react';
import useSound from 'use-sound';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

import recordSfx from '@/assets/cassette_tape_record.mp3';

interface SystemOutputToggleProps {
  isActive: boolean;
  onClick: () => void;
}

export function SystemOutputToggle({
  isActive,
  onClick
}: SystemOutputToggleProps) {
  const [play] = useSound(recordSfx);
  return (
    <Button
      variant="outline"
      size="sm"
      onClick={() => {
        onClick();
        play();
      }}
      className={cn(
        'transition-colors',
        isActive &&
          'border-red-500 text-red-800 animate-recording-pulse bg-red-100 dark:bg-red-950'
      )}
    >
      <Speaker className="h-[1.2rem] w-[1.2rem]" />
      <span>{isActive ? 'Stop recording' : 'Use system output'}</span>
    </Button>
  );
}

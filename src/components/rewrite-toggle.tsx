import { PenLine } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface RewriteToggleProps {
  isActive: boolean;
  onClick: () => void;
}

export function RewriteToggle({ isActive, onClick }: RewriteToggleProps) {
  return (
    <Button
      variant="outline"
      size="icon"
      onClick={onClick}
      className={cn(
        'transition-colors',
        isActive && 'border-green-500 bg-green-50 dark:hover:bg-green-950'
      )}
    >
      <PenLine
        className="h-[1.2rem] w-[1.2rem]"
        color={isActive ? 'green' : 'currentColor'}
      />
      <span className="sr-only">Toggle rewrite mode</span>
    </Button>
  );
}

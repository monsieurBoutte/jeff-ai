import { useEffect, useRef, useState } from "react";

interface DropdownPosition<T extends HTMLElement> {
	ref: React.RefObject<T>;
	maxHeight: number;
	side: "top" | "bottom";
}

interface UseDropdownPositionProps {
	margin?: number;
	updateOnScroll?: boolean;
}

export function useDropdownPosition<T extends HTMLElement>({
	margin,
	updateOnScroll,
}: UseDropdownPositionProps = {}): DropdownPosition<T> {
	const ref = useRef<T>(null);
	const [maxHeight, setMaxHeight] = useState<number>(300); // Default max height
	const [side, setSide] = useState<"top" | "bottom">("bottom");

	useEffect(() => {
		function updatePosition() {
			if (ref.current) {
				const rect = ref.current.getBoundingClientRect();
				const windowHeight = window.innerHeight;
				const spaceBelow = windowHeight - rect.bottom;
				const spaceAbove = rect.top;

				if (spaceBelow >= spaceAbove) {
					setSide("bottom");
					setMaxHeight(margin ? spaceBelow - margin : spaceBelow);
				} else {
					setSide("top");
					setMaxHeight(margin ? spaceAbove - margin : spaceAbove);
				}
			}
		}

		updatePosition();

		window.addEventListener("resize", updatePosition);

		if (updateOnScroll) {
			window.addEventListener("scroll", updatePosition, true);
		}

		return () => {
			window.removeEventListener("resize", updatePosition);
			if (updateOnScroll) {
				window.removeEventListener("scroll", updatePosition, true);
			}
		};
	}, [margin, updateOnScroll]);

	return { ref, maxHeight, side };
}


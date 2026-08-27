import { createContext } from "react";

export interface SliderContextType {
	activeIndex: number;
	setActiveIndex: (index: number) => void;
	floatIndex: number;
	isDragging: boolean;
	isSwipeDisabled: boolean;
	setIsSwipeDisabled: (disabled: boolean) => void;
}

export const SliderContext = createContext<SliderContextType | null>(null);
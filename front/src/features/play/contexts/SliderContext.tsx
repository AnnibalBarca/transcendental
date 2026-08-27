import { useContext } from "react";
import { SliderContext, type SliderContextType } from "./slider-context";

export function SliderProvider({
	children,
	value,
}: {
	children: React.ReactNode;
	value: SliderContextType;
}) {
	return (
		<SliderContext.Provider value={value}>{children}</SliderContext.Provider>
	);
}

export function useSlider() {
	const context = useContext(SliderContext);
	if (!context) {
		throw new Error("useSlider must be used within a SliderProvider");
	}
	return context;
}
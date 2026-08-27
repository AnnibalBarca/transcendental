import type { ReactNode } from "react";

interface SliderSectionProps {
	children: ReactNode;
	className?: string;
}

export function SliderSection({
	children,
	className = "",
}: SliderSectionProps) {
	return (
		<section
			className={`relative z-[1] flex h-full w-full flex-col items-center justify-center ${className}`}
		>
			{children}
		</section>
	);
}

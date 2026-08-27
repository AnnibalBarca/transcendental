import { useCallback, useEffect, useState } from "react";
import useEmblaCarousel from "embla-carousel-react";

export interface UseEmblaSliderOptions {
	slideCount: number;
	initialIndex?: number;
	dragThreshold?: number;
}

export interface UseEmblaSliderReturn {
	activeIndex: number;
	setActiveIndex: (index: number) => void;
	floatIndex: number;
	isDragging: boolean;
	isSwipeDisabled: boolean;
	setIsSwipeDisabled: (disabled: boolean) => void;
	emblaRef: (node: HTMLElement | null) => void;
}

export function useEmblaSlider({
	slideCount,
	initialIndex = 0,
	dragThreshold = 25,
}: UseEmblaSliderOptions): UseEmblaSliderReturn {
	const [emblaRef, emblaApi] = useEmblaCarousel({
		loop: false,
		startIndex: initialIndex,
		dragThreshold,
	});

	const [activeIndex, setActiveIndexState] = useState(initialIndex);
	const [floatIndex, setFloatIndex] = useState(initialIndex);
	const [isDragging, setIsDragging] = useState(false);
	const [isSwipeDisabled, setIsSwipeDisabled] = useState(false);

	useEffect(() => {
		if (!emblaApi) return;

		const onSelect = () => {
			setActiveIndexState(emblaApi.selectedScrollSnap());
		};

		const onPointerDown = () => setIsDragging(true);
		const onPointerUp = () => setIsDragging(false);
		const onSettle = () => setIsDragging(false);

		emblaApi.on("select", onSelect);
		emblaApi.on("pointerDown", onPointerDown);
		emblaApi.on("pointerUp", onPointerUp);
		emblaApi.on("settle", onSettle);

		setActiveIndexState(emblaApi.selectedScrollSnap());

		return () => {
			emblaApi.off("select", onSelect);
			emblaApi.off("pointerDown", onPointerDown);
			emblaApi.off("pointerUp", onPointerUp);
			emblaApi.off("settle", onSettle);
		};
	}, [emblaApi]);

	useEffect(() => {
		if (!emblaApi) return;

		const onScroll = () => {
			const progress = emblaApi.scrollProgress();
			setFloatIndex(progress * (slideCount - 1));
		};

		emblaApi.on("scroll", onScroll);
		return () => {
			emblaApi.off("scroll", onScroll);
		};
	}, [emblaApi, slideCount]);

	useEffect(() => {
		if (!emblaApi) return;
		emblaApi.reInit({ watchDrag: !isSwipeDisabled });
	}, [emblaApi, isSwipeDisabled]);

	const setActiveIndex = useCallback(
		(index: number) => {
			emblaApi?.scrollTo(Math.max(0, Math.min(slideCount - 1, index)));
		},
		[emblaApi, slideCount],
	);

	return {
		activeIndex,
		setActiveIndex,
		floatIndex,
		isDragging,
		isSwipeDisabled,
		setIsSwipeDisabled,
		emblaRef,
	};
}

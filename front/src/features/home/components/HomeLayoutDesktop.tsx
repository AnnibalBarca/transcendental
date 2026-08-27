import { Children, useRef, type ReactNode } from "react";
import { DesktopNavbar } from "@/features/play/components/DesktopNavbar";
import FriendPanel from "@/features/play/components/FriendPanel";
import { SliderProvider } from "@/features/play/contexts/SliderContext";
import { useEmblaSlider } from "@/features/play/hooks/useEmblaSlider";
import { useDragToScroll } from "@/features/play/hooks/useDragToScroll";
import { useWallet } from "@/features/play/hooks/useWallet";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { HomeBackground } from "@/components/HomeBackground";

interface HomeLayoutDesktopProps {
	children: ReactNode;
}

const SLIDE_PAGES = [2, 1, 0, 4];
const PAGE_TO_SLIDE: Record<number, number> = { 2: 0, 1: 1, 0: 2, 4: 3 };

function DesktopSlide({ children }: { children: ReactNode }) {
	const ref = useRef<HTMLDivElement>(null);
	useDragToScroll(ref);

	return (
		<div
			ref={ref}
			className="h-full min-w-0 flex-[0_0_100%]"
		>
			{children}
		</div>
	);
}

export default function HomeLayoutDesktop({
	children,
}: HomeLayoutDesktopProps) {
	const swipe = useEmblaSlider({ slideCount: SLIDE_PAGES.length, initialIndex: 0 });
	const wallet = useWallet();
	const { user } = useAuth();

	const pages = Children.toArray(children);

	const activePage = SLIDE_PAGES[swipe.activeIndex] ?? 2;

	const setActivePage = (pageIndex: number) => {
		const slide = PAGE_TO_SLIDE[pageIndex];
		if (slide !== undefined) swipe.setActiveIndex(slide);
	};

	const sliderValue = {
		activeIndex: activePage,
		setActiveIndex: setActivePage,
		floatIndex: activePage,
		isDragging: swipe.isDragging,
		isSwipeDisabled: swipe.isSwipeDisabled,
		setIsSwipeDisabled: swipe.setIsSwipeDisabled,
	};

	return (
		<SliderProvider value={sliderValue}>
			<div className="relative flex h-screen w-screen flex-col overflow-hidden bg-[#080b14] text-white supports-[height:100dvh]:h-dvh">
				<HomeBackground />
				<DesktopNavbar
					activeIndex={activePage}
					setActiveIndex={setActivePage}
					level={user?.level}
					progress={user?.xp_progress}
					xp={user?.xp}
					wallet={wallet}
				/>

				<div className="min-h-0 flex-1 overflow-hidden" ref={swipe.emblaRef}>
					<div className="flex h-full">
						{SLIDE_PAGES.map((pageIndex) => (
							<DesktopSlide key={pageIndex}>
								{pageIndex === 2 ? (
									<div className="h-full w-full overflow-y-auto">
										<div className="flex min-h-full w-full items-start justify-center gap-[10%] px-8 py-8">
											<div className="min-w-0 flex-[1_1_0%] max-w-[680px]">
												{pages[2]}
											</div>
											<div className="mt-20 w-[380px] shrink-0">
												<FriendPanel />
											</div>
										</div>
									</div>
								) : (
									<div className="h-full w-full">
										{pages[pageIndex]}
									</div>
								)}
							</DesktopSlide>
						))}
					</div>
				</div>
			</div>
		</SliderProvider>
	);
}

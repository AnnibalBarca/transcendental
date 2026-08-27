import { Children, type ReactNode } from "react";
import { Navbar } from "@/features/play/components/Navbar";
import { Levels } from "@/features/play/components/Levels";
import { Wallet } from "@/features/play/components/Wallet";
import { SliderProvider } from "@/features/play/contexts/SliderContext";
import { useEmblaSlider } from "@/features/play/hooks/useEmblaSlider";
import { useIsDesktop } from "@/features/play/hooks/useMediaQuery";
import { useWallet } from "@/features/play/hooks/useWallet";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { HomeBackground } from "@/components/HomeBackground";
import HomeLayoutDesktop from "./HomeLayoutDesktop";

interface HomeLayoutProps {
	children: ReactNode;
}

function HomeLayoutMobile({ children }: HomeLayoutProps) {
	const swipe = useEmblaSlider({ slideCount: 5, initialIndex: 2 });
	const wallet = useWallet();
	const { user } = useAuth();

	const showTopBar =
		swipe.activeIndex === 0 ||
		swipe.activeIndex === 1 ||
		swipe.activeIndex === 2;

	return (
		<SliderProvider value={swipe}>
			<div className="relative flex h-screen w-screen flex-col overflow-hidden text-white supports-[height:100dvh]:h-dvh">
				<HomeBackground />

				<div
					className={`absolute top-0 right-0 left-0 z-20 flex justify-center px-4 pt-4 transition-all duration-300 ${
						showTopBar
							? "translate-y-0 opacity-100"
							: "pointer-events-none -translate-y-4 opacity-0"
					}`}
				>
					<div className="flex w-full max-w-[680px] items-center justify-between">
						<Levels level={user?.level} progress={user?.xp_progress} xp={user?.xp} />
						<Wallet balance={wallet ?? 0} />
					</div>
				</div>

				<div
					className="relative z-[1] min-h-0 flex-1 overflow-hidden"
					ref={swipe.emblaRef}
				>
					<div className="flex h-full">
						{Children.map(children, (child, index) => (
							<div className="h-full min-w-0 flex-[0_0_100vw]" key={index}>
								{child}
							</div>
						))}
					</div>
				</div>

				<div style={{ pointerEvents: swipe.isSwipeDisabled ? "none" : "auto" }}>
					<Navbar
						activeIndex={swipe.activeIndex}
						setActiveIndex={swipe.setActiveIndex}
						floatIndex={swipe.floatIndex}
					/>
				</div>
			</div>
		</SliderProvider>
	);
}

export default function HomeLayout({ children }: HomeLayoutProps) {
	const isDesktop = useIsDesktop();
	return isDesktop ? (
		<HomeLayoutDesktop>{children}</HomeLayoutDesktop>
	) : (
		<HomeLayoutMobile>{children}</HomeLayoutMobile>
	);
}

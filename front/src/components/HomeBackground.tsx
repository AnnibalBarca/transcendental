import { useContext, useEffect, useRef, useState } from "react";
import { SliderContext } from "@/features/play/contexts/slider-context";

const IMG_BASE = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "/img";

interface HomeBackgroundProps {
	className?: string;
}

// iOS 13+ expose DeviceOrientationEvent.requestPermission, pas dans les autres navigateurs
type DeviceOrientationEventiOS = typeof DeviceOrientationEvent & {
	requestPermission?: () => Promise<"granted" | "denied">;
};

export function HomeBackground({ className = "" }: HomeBackgroundProps) {
	const { floatIndex = 0 } = useContext(SliderContext) ?? {};
	const leftRef = useRef<HTMLImageElement>(null);
	const centerRef = useRef<HTMLImageElement>(null);
	const rightRef = useRef<HTMLImageElement>(null);
	const tilt = useRef({ x: 0.5, y: 0.5 });
	const slide = useRef(floatIndex);
	const initialSlide = useRef(floatIndex);
	const reduced = useRef(false);
	const [needsIOSPermission, setNeedsIOSPermission] = useState(false);

	useEffect(() => {
		slide.current = floatIndex;
	}, [floatIndex]);

	useEffect(() => {
		reduced.current = window.matchMedia(
			"(prefers-reduced-motion: reduce)",
		).matches;

		const isTouchDevice = window.matchMedia("(pointer: coarse)").matches;

		const onMove = (e: MouseEvent) => {
			tilt.current.x = e.clientX / window.innerWidth;
			tilt.current.y = e.clientY / window.innerHeight;
		};

		const onOrientation = (e: DeviceOrientationEvent) => {
			// gamma: inclinaison gauche/droite (-90 à 90), beta: avant/arrière (-180 à 180)
			if (e.gamma === null || e.beta === null) return;
			const gamma = Math.max(-45, Math.min(45, e.gamma));
			const beta = Math.max(-45, Math.min(45, e.beta - 45)); // offset pour tenir tel légèrement incliné en usage normal
			tilt.current.x = (gamma + 45) / 90;
			tilt.current.y = (beta + 45) / 90;
		};

		if (isTouchDevice && "DeviceOrientationEvent" in window) {
			const DOE = DeviceOrientationEvent as DeviceOrientationEventiOS;
			if (typeof DOE.requestPermission === "function") {
				// iOS: permission nécessaire, doit venir d'un geste utilisateur
				setNeedsIOSPermission(true);
			} else {
				// Android / navigateurs sans permission API : direct
				window.addEventListener("deviceorientation", onOrientation);
			}
		} else {
			window.addEventListener("mousemove", onMove);
		}

		return () => {
			window.removeEventListener("mousemove", onMove);
			window.removeEventListener("deviceorientation", onOrientation);
		};
	}, []);

	const requestIOSPermission = async () => {
		const DOE = DeviceOrientationEvent as DeviceOrientationEventiOS;
		if (typeof DOE.requestPermission !== "function") return;
		try {
			const result = await DOE.requestPermission();
			if (result === "granted") {
				setNeedsIOSPermission(false);
				window.addEventListener("deviceorientation", (e) => {
					if (e.gamma === null || e.beta === null) return;
					const gamma = Math.max(-45, Math.min(45, e.gamma));
					const beta = Math.max(-45, Math.min(45, e.beta - 45));
					tilt.current.x = (gamma + 45) / 90;
					tilt.current.y = (beta + 45) / 90;
				});
			}
		} catch {
		}
	};

	useEffect(() => {
		if (reduced.current) return;

		let raf = 0;
		const tick = () => {
			const mx = (tilt.current.x - 0.5) * 2;
			const my = (tilt.current.y - 0.5) * 2;
			const slideShift = slide.current - initialSlide.current;

			if (leftRef.current) {
				leftRef.current.style.transform = `translate3d(${(
					-mx * 45 +
					slideShift * 35
				).toFixed(2)}px, ${(my * -28).toFixed(2)}px, 0)`;
			}
			if (centerRef.current) {
				centerRef.current.style.transform = `translate3d(${(
					-mx * 14 +
					slideShift * 14
				).toFixed(2)}px, ${(my * -14).toFixed(2)}px, 0)`;
			}
			if (rightRef.current) {
				rightRef.current.style.transform = `translate3d(${(
					-mx * 55 +
					slideShift * 35
				).toFixed(2)}px, ${(my * -32).toFixed(2)}px, 0)`;
			}

			raf = requestAnimationFrame(tick);
		};

		raf = requestAnimationFrame(tick);
		return () => cancelAnimationFrame(raf);
	}, []);

	return (
		<div
			aria-hidden={!needsIOSPermission}
			className={`pointer-events-none absolute inset-0 z-0 overflow-hidden bg-black ${className}`}
		>
			<div className="absolute -top-[18vh] -left-[18vw] h-[70vmax] w-[70vmax] rounded-full bg-[radial-gradient(circle,rgba(34,211,238,0.18),transparent_62%)]" />
			<div className="absolute -right-[18vw] -bottom-[22vh] h-[62vmax] w-[62vmax] rounded-full bg-[radial-gradient(circle,rgba(139,92,246,0.18),transparent_62%)]" />

			{needsIOSPermission && (
				<button
					type="button"
					onClick={requestIOSPermission}
					className="pointer-events-auto absolute top-4 right-4 z-30 rounded-full bg-white/10 px-3 py-1.5 text-xs text-white/80 backdrop-blur"
				>
					Activer l'effet 3D
				</button>
			)}

			<img
				ref={leftRef}
				src={`${IMG_BASE}/carte-compress/beast_of_burden_0.svg`}
				alt=""
				className="absolute top-1/2 left-[-65vw] z-20 w-[max(130vw,66.67dvh)] max-w-[1400px] -translate-y-1/2 -rotate-12 opacity-20 lg:left-[-25vw] lg:opacity-45"
			/>
			<img
				ref={centerRef}
				src={`${IMG_BASE}/carte-compress/bestification_0.svg`}
				alt=""
				className="absolute top-1/2 left-1/2 z-10 w-[max(130vw,66.67dvh)] max-w-[800px] -translate-x-1/2 -translate-y-1/2 opacity-70 lg:opacity-90"
			/>
			<img
				ref={rightRef}
				src={`${IMG_BASE}/carte-compress/hermit_architect_0.svg`}
				alt=""
				className="absolute top-1/2 right-[-65vw] z-20 w-[max(130vw,66.67dvh)] max-w-[1400px] -translate-y-1/2 rotate-12 opacity-20 lg:right-[-25vw] lg:opacity-45"
			/>
		</div>
	);
}
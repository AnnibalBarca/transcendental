import { useEffect, useRef, useState } from "react";

type DeviceOrientationEventiOS = typeof DeviceOrientationEvent & {
	requestPermission?: () => Promise<"granted" | "denied">;
};

export function useCardTilt<T extends HTMLElement>(active: boolean) {
	const elRef = useRef<T | null>(null);
	const tilt = useRef({ cx: 0, cy: 0, px: 0.5, py: 0.5 });
	const isTouch = useRef(false);
	const [needsIOSPermission, setNeedsIOSPermission] = useState(false);

	const applyOrientation = (e: DeviceOrientationEvent) => {
		if (e.gamma === null || e.beta === null) return;
		const gamma = Math.max(-35, Math.min(35, e.gamma));
		const beta = Math.max(-35, Math.min(35, e.beta - 35)); // offset tel légèrement incliné
		tilt.current.cx = gamma / 40; // -0.5..0.5, même échelle que la souris
		tilt.current.cy = beta / 40;
		tilt.current.px = tilt.current.cx + 0.5;
		tilt.current.py = tilt.current.cy + 0.5;
	};

	useEffect(() => {
		isTouch.current = window.matchMedia("(pointer: coarse)").matches;
		if (!isTouch.current || !("DeviceOrientationEvent" in window)) return;

		const DOE = DeviceOrientationEvent as DeviceOrientationEventiOS;
		if (typeof DOE.requestPermission === "function") {
			setNeedsIOSPermission(true);
		} else {
			window.addEventListener("deviceorientation", applyOrientation);
		}

		return () => window.removeEventListener("deviceorientation", applyOrientation);
	}, []);

	const requestIOSPermission = async () => {
		const DOE = DeviceOrientationEvent as DeviceOrientationEventiOS;
		if (typeof DOE.requestPermission !== "function") return;
		try {
			const result = await DOE.requestPermission();
			if (result === "granted") {
				setNeedsIOSPermission(false);
				window.addEventListener("deviceorientation", applyOrientation);
			}
		} catch {}
	};

	const handleMouseMove = (e: React.MouseEvent<T>) => {
		if (isTouch.current) return;
		const rect = elRef.current?.getBoundingClientRect();
		if (!rect) return;
		const px = (e.clientX - rect.left) / rect.width;
		const py = (e.clientY - rect.top) / rect.height;
		tilt.current = { cx: px - 0.5, cy: py - 0.5, px, py };
	};

	const handleMouseLeave = () => {
		if (isTouch.current) return;
		tilt.current = { cx: 0, cy: 0, px: 0.5, py: 0.5 };
	};

	useEffect(() => {
		if (!active) return;
		let raf = 0;
		const tick = () => {
			const el = elRef.current;
			if (el) {
				const { cx, cy, px, py } = tilt.current;
				const s = el.style;
				s.setProperty("--mx", `${(px * 100).toFixed(2)}%`);
				s.setProperty("--my", `${(py * 100).toFixed(2)}%`);
				s.setProperty("--posx", `${(px * 100).toFixed(2)}%`);
				s.setProperty("--posy", `${(py * 100).toFixed(2)}%`);
				s.setProperty("--rx", `${(cy * 28).toFixed(2)}deg`);
				s.setProperty("--ry", `${(cx * 28).toFixed(2)}deg`);
				s.setProperty("--hyp", Math.min(1, Math.hypot(cx, cy) * 3).toFixed(3));
			}
			raf = requestAnimationFrame(tick);
		};
		raf = requestAnimationFrame(tick);
		return () => cancelAnimationFrame(raf);
	}, [active]);

	return { elRef, needsIOSPermission, requestIOSPermission, handleMouseMove, handleMouseLeave };
}
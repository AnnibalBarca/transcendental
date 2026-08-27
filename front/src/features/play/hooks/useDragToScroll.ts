import { useEffect, type RefObject } from "react";

function findScrollable(
	start: HTMLElement,
	root: HTMLElement,
): HTMLElement | null {
	let el: HTMLElement | null = start;
	while (el && root.contains(el)) {
		if (el.scrollHeight > el.clientHeight) return el;
		el = el.parentElement;
	}
	return null;
}

export function useDragToScroll(
	ref: RefObject<HTMLElement | null>,
	enabled = true,
) {
	useEffect(() => {
		const el = ref.current;
		if (!el || !enabled) return;

		let scrollEl: HTMLElement | null = null;
		let startX = 0;
		let startY = 0;
		let startScroll = 0;
		let active = false;
		let lastTime = 0;
		let lastScroll = 0;
		let velocity = 0;
		let rafId = 0;

		const stopInertia = () => {
			if (rafId) {
				cancelAnimationFrame(rafId);
				rafId = 0;
			}
			velocity = 0;
		};

		const onPointerDown = (e: PointerEvent) => {
			if (e.pointerType !== "mouse") return;
			if (e.button !== 0) return;
			const target = e.target as HTMLElement;
			scrollEl = findScrollable(target, el);
			if (!scrollEl) return;
			stopInertia();
			startX = e.clientX;
			startY = e.clientY;
			startScroll = scrollEl.scrollTop;
			lastTime = performance.now();
			lastScroll = startScroll;
			velocity = 0;
			active = false;
		};

		const onPointerMove = (e: PointerEvent) => {
			if (!scrollEl || e.pointerType !== "mouse") return;
			const dx = e.clientX - startX;
			const dy = e.clientY - startY;

			if (!active) {
				if (Math.abs(dy) > 6 && Math.abs(dy) > Math.abs(dx)) {
					active = true;
					el.style.userSelect = "none";
				} else {
					return;
				}
			}

			e.preventDefault();
			e.stopPropagation();
			const next = startScroll - dy;
			scrollEl.scrollTop = next;

			const now = performance.now();
			const dt = now - lastTime;
			const dScroll = next - lastScroll;
			if (dt > 0) {
				const inst = dScroll / dt;
				velocity = inst * 0.3 + velocity * 0.7;
			}
			lastTime = now;
			lastScroll = next;
		};

		const onPointerUp = () => {
			if (!scrollEl) return;
			const target = scrollEl;
			const startVel = velocity;
			const decay = 0.96;
			const minVel = 0.08;
			let prev = target.scrollTop;

			scrollEl = null;
			active = false;
			el.style.userSelect = "";
			velocity = 0;

			if (Math.abs(startVel) < minVel) return;

			const maxScroll = target.scrollHeight - target.clientHeight;

			const step = () => {
				velocity *= decay;
				if (Math.abs(velocity) < minVel) {
					rafId = 0;
					return;
				}
				const next = target.scrollTop + velocity;
				if (next < 0 || next > maxScroll || next === prev) {
					rafId = 0;
					return;
				}
				prev = next;
				target.scrollTop = next;
				rafId = requestAnimationFrame(step);
			};

			rafId = requestAnimationFrame(step);
		};

		const onPointerCancel = () => {
			stopInertia();
			scrollEl = null;
			active = false;
			el.style.userSelect = "";
		};

		el.addEventListener("pointerdown", onPointerDown);
		el.addEventListener("pointermove", onPointerMove);
		el.addEventListener("pointerup", onPointerUp);
		el.addEventListener("pointercancel", onPointerCancel);

		return () => {
			el.removeEventListener("pointerdown", onPointerDown);
			el.removeEventListener("pointermove", onPointerMove);
			el.removeEventListener("pointerup", onPointerUp);
			el.removeEventListener("pointercancel", onPointerCancel);
			stopInertia();
		};
	}, [ref, enabled]);
}

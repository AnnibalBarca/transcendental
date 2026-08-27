import { useState, useRef, useEffect } from "react";

const CELL_SIZE = 60;
const SPOT_RADIUS = 150;

const NOISE_BG =
	"url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='80' height='80'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.6'/%3E%3C/svg%3E\")";

interface MousePos {
  x: number;
  y: number;
}

export default function NeonSpotGrid() {
  const [mouse, setMouse] = useState<MousePos | null>(null);
  const [size, setSize] = useState({ width: 0, height: 0 });
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const observer = new ResizeObserver(([entry]) => {
      if (entry) setSize(entry.contentRect);
    });

    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const handleWindowMove = (e: MouseEvent) => {
      if (!ref.current) return;
      const rect = ref.current.getBoundingClientRect();
      setMouse({ x: e.clientX - rect.left, y: e.clientY - rect.top });
    };

    window.addEventListener("mousemove", handleWindowMove);
    return () => window.removeEventListener("mousemove", handleWindowMove);
  }, []);

  const COLS = Math.max(1, Math.round(size.width / CELL_SIZE));
  const ROWS = Math.max(1, Math.round(size.height / CELL_SIZE));

  const maskImage = mouse
    ? `radial-gradient(circle ${SPOT_RADIUS}px at ${mouse.x}px ${mouse.y}px, black 0%, black 15%, transparent 100%)`
    : "none";

  const colsArray = Array.from({ length: COLS + 1 });
  const rowsArray = Array.from({ length: ROWS + 1 });

  const LINE_VERTICAL_BASE =
    "absolute top-0 bottom-0 w-px bg-[rgba(255,255,255,0.06)]";
  const LINE_HORIZONTAL_BASE =
    "absolute left-0 right-0 h-px bg-[rgba(255,255,255,0.06)]";
  const LINE_VERTICAL_GLOW =
    "absolute top-0 bottom-0 w-px bg-[#dc2626] shadow-[0_0_6px_#dc2626,0_0_12px_#dc2626]";
  const LINE_HORIZONTAL_GLOW =
    "absolute left-0 right-0 h-px bg-[#dc2626] shadow-[0_0_6px_#dc2626,0_0_12px_#dc2626]";

  return (
    <div className="absolute inset-0 overflow-hidden z-0">
      <div ref={ref} className="relative w-full h-full">

        <div
          aria-hidden
          className="absolute inset-0 opacity-25 mix-blend-overlay"
          style={{ backgroundImage: NOISE_BG, backgroundSize: "80px 80px" }}
        />

        {colsArray.map((_, i) => (
          <div key={`v-base-${i}`} className={LINE_VERTICAL_BASE} style={{ left: `${(i / COLS) * 100}%` }} />
        ))}
        {rowsArray.map((_, j) => (
          <div key={`h-base-${j}`} className={LINE_HORIZONTAL_BASE} style={{ top: `${(j / ROWS) * 100}%` }} />
        ))}

        <div
          className="absolute inset-0 pointer-events-none"
          style={{ WebkitMaskImage: maskImage, maskImage }}
        >
          {colsArray.map((_, i) => (
            <div key={`v-glow-${i}`} className={LINE_VERTICAL_GLOW} style={{ left: `${(i / COLS) * 100}%` }} />
          ))}
          {rowsArray.map((_, j) => (
            <div key={`h-glow-${j}`} className={LINE_HORIZONTAL_GLOW} style={{ top: `${(j / ROWS) * 100}%` }} />
          ))}
        </div>

      </div>
    </div>
  );
}
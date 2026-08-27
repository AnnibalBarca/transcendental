import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cn } from "@/lib/utils";

const IMG_BASE = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "/img";
const DEFAULT_TEXTURE = `url("${IMG_BASE}/carte/breakthrough_0.svg")`;

interface SquareButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
	children?: ReactNode;
	tone?: "blue" | "red";
	texture?: string;
	textureZoom?: number;
	texturePosition?: string;
	textureOpacity?: number;
}

export function SquareButton({
	children,
	tone = "blue",
	texture = DEFAULT_TEXTURE,
	textureZoom = 500,
	texturePosition = "center 20%",
	textureOpacity = 0.5,
	className = "",
	...props
}: SquareButtonProps) {
	const bgImage = texture.startsWith("url(") ? texture : `url("${texture}")`;

	return (
		<button
			type="button"
			{...props}
			className={cn(
				"relative flex h-14 w-14 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-[2px] font-semibold text-white hover:brightness-95 active:brightness-90 disabled:pointer-events-none disabled:opacity-50",
				tone === "red"
					? "bg-linear-to-br from-red-900 via-red-950 to-slate-950"
					: "bg-linear-to-br from-blue-900 via-blue-950 to-slate-950",
				className,
			)}
		>
			<span
				aria-hidden
				className="pointer-events-none absolute inset-0 mix-blend-screen"
				style={{
					backgroundImage: bgImage,
					backgroundSize: `${textureZoom}%`,
					backgroundPosition: texturePosition,
					filter: "grayscale(100%)",
					opacity: textureOpacity,
				}}
			/>
			<span className="relative flex items-center justify-center gap-2">
				{children}
			</span>
		</button>
	);
}
import type { CSSProperties } from "react";
import { PROFILE_SLOTS, deserializeProfilePicture, hasProfilePicture } from "@/utils/profilePicture";

interface ProfilePictureProps {
	pictureId?: string | null;
	size?: number;
	className?: string;
	style?: CSSProperties;
}

export default function ProfilePicture({
	pictureId,
	size = 40,
	className,
	style,
}: ProfilePictureProps) {
	const equipped = deserializeProfilePicture(pictureId ?? "");

	if (!hasProfilePicture(pictureId)) {
		return (
			<div
				className={className}
				style={{
					width: size,
					height: size,
					borderRadius: size / 4,
					background: "linear-gradient(to bottom right, #06b6d4, #0891b2)",
					boxShadow: "inset 0 0 0 1px rgba(255,255,255,0.12)",
					...style,
				}}
				aria-hidden="true"
			/>
		);
	}

	return (
		<svg
			viewBox="0 0 1000 1000"
			width={size}
			height={size}
			className={className}
			style={{ display: "block", borderRadius: size / 4, ...style }}
			aria-hidden="true"
		>
			{PROFILE_SLOTS.map((slot) => {
				const item = equipped[slot];
				if (!item) return null;
				return (
					<image key={slot} href={item.image} width="1000" height="1000" />
				);
			})}
		</svg>
	);
}

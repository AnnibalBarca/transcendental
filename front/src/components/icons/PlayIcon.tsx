import type { SVGProps } from "react";

const PlayIcon = (props: SVGProps<SVGSVGElement>) => (
	<svg
		{...props}
		stroke="currentColor"
		strokeWidth={3}
		viewBox="0 0 16 16"
		xmlns="http://www.w3.org/2000/svg"
		width={props.width || 24}
		height={props.height || 24}
	>
		<path fill={props.fill || "#ffffff"} d="m1 0 14 8 -14 8V0Z" strokeWidth={0} />
	</svg>
);

export { PlayIcon };

import type { SVGProps } from "react";

const Hamburger = (props: SVGProps<SVGSVGElement>) => (
	<svg
		{...props}
		fill="none"
		stroke="currentColor"
		strokeWidth={3}
		viewBox="0 0 24 24"
		xmlns="http://www.w3.org/2000/svg"
		width={props.width || 24}
		height={props.height || 24}
	>	
		<path strokeLinecap="round" strokeLinejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
	</svg>

);

export { Hamburger };

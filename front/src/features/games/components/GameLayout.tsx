import styles from '@/features/games/components/styles/GameLayout.module.css'
import { HomeBackground } from "@/components/HomeBackground";

export default function GameLayout({ children }: { children: React.ReactNode })
{
	return (
		<div className={styles.gameLayout}>
			<HomeBackground />
			<div className={styles.content}>
				{children}
			</div>
		</div>
	);
}
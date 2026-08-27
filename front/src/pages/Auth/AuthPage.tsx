import AuthLayout from '@/features/auth/components/AuthLayout'
import AuthForm from '@/features/auth/components/AuthForm'
import AuthLeftSidebar from '@/features/auth/components/AuthLeftSidebar'
import { useLocation } from 'react-router-dom'

export default function AuthPage() {
	const location = useLocation();

	const mode = location.pathname === "/signup" ? "signup" : "signin";

	return (
		<AuthLayout>
			<div className="flex flex-row items-center w-full">
				<AuthLeftSidebar />
				<AuthForm key={mode} mode={mode} />
			</div>
		</AuthLayout>
	);
}
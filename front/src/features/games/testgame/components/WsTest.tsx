import { useState } from "react";
import { useGameSocketSimple } from "../hooks/useGameSocketSimple";
import styles from "./styles/WsTest.module.css";

const DEFAULT_JSON = '{\n  "action": "ping"\n}';

export default function WsTest() {
	const { connected, error, messages, sendJson, clearMessages } =
		useGameSocketSimple();
	const [input, setInput] = useState(DEFAULT_JSON);

	return (
		<div className={styles.container}>
			<h1>WebSocket Test</h1>

			<div className={styles.status}>
				<span
					className={`${styles.dot} ${
						connected ? styles.dotOnline : styles.dotOffline
					}`}
				/>
				{connected ? "Connected" : "Disconnected"}
				{error && <span className={styles.error}> — {error}</span>}
			</div>

			<div className={styles.inputRow}>
				<textarea
					className={styles.textarea}
					value={input}
					onChange={(e) => setInput(e.target.value)}
					rows={6}
				/>
				<button
					className={styles.sendBtn}
					onClick={() => sendJson(input)}
					disabled={!connected}
				>
					Send
				</button>
			</div>

			<div className={styles.messagesHeader}>
				<h2>Messages received ({messages.length})</h2>
				<button className={styles.clearBtn} onClick={clearMessages}>
					Clear
				</button>
			</div>

			<div className={styles.messages}>
				{messages.length === 0 && (
					<p className={styles.empty}>No messages yet.</p>
				)}
				{messages.map((msg, i) => (
					<pre key={i} className={styles.message}>
						{msg}
					</pre>
				))}
			</div>
		</div>
	);
}

interface StatusCircleProps {
    status: "on" | "off";
}

const StatusCircle: React.FC<StatusCircleProps> = ({ status }) => {
    return <div className={`status-circle ${status}`}> {/* スタイル適用 */}</div>;
};

export default StatusCircle;

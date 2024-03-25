"use client";

import { useEffect, useState } from "react";

type Props = {
    locale?: string;
};

const ShowDate = ({ locale = "ja-JP" }: Props) => {
    const [date, setDate] = useState(new Date());

    useEffect(() => {
        const timer = setInterval(() => {
            setDate(new Date());
        }, 1000);

        return () => {
            clearInterval(timer);
        };
    }, []);

    return <div className="block p-2">{date.toLocaleString(locale)}</div>;
};

export default ShowDate;

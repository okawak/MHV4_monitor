"use client";

import React, { useState } from "react";

interface InputProps {
  inputs: number[];
}

const ApplyButton: React.FC<InputProps> = ({ inputs }) => {
  const [loading, setLoading] = useState(false);

  const handleSubmit = async () => {
    setLoading(true);
    console.log("input value:", inputs);
    try {
      const response = await fetch(`${process.env.NEXT_PUBLIC_HV_ROUTE}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(inputs),
      });
      if (!response.ok) {
        throw new Error(`Error: ${response.status}`);
      }
      const responseData = await response.json();
      console.log("Result from apply route:", responseData);
    } catch (error) {
      console.error("Fetch error:", error);
    } finally {
      setLoading(false);
    }
  };

  let style_str = "inline-flex items-center justify-center";
  style_str += " whitespace-nowrap rounded-md text-sm font-medium";
  style_str += " ring-offset-background transition-colors";
  style_str += " focus-visible:outline-none focus-visible:ring-2";
  style_str += " focus-visible:ring-ring focus-visible:ring-offset-2";
  style_str += " disabled:pointer-events-none disabled:opacity-50";

  style_str += " bg-green-700 text-primary-foreground hover:bg-green-700/80";
  style_str += " mx-1 h-8 px-4 py-2";
  return (
    <div className={style_str}>
      <button onClick={handleSubmit} disabled={loading}>
        {loading ? "Sending..." : "apply HV"}
      </button>
    </div>
  );
};

export default ApplyButton;

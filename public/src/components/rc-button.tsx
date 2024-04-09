"use client";

import React, { useState, useContext } from "react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { useMHV4Data } from "@/contexts/MHV4Context";

const RCButton: React.FC = () => {
  const { rcType, setRCType } = useMHV4Data();
  const [loading, setLoading] = useState(false);

  const handleSubmit = async () => {
    setLoading(true);
    console.log("input value: RC mode is >", rcType);
    try {
      const response = await fetch(`${process.env.NEXT_PUBLIC_STATUS_ROUTE}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(!rcType),
      });
      if (!response.ok) {
        throw new Error(`Error: ${response.status}`);
      }
      const responseData = await response.json();
      console.log("Result from apply route:", responseData);
      if (responseData) {
        setRCType(!rcType);
      }
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

  if (rcType) {
    style_str += " bg-green-700 text-primary-foreground hover:bg-green-700/80";
  } else {
    style_str += " bg-red-700 text-primary-foreground hover:bg-red-700/80";
  }
  style_str += " mx-1 h-8 px-4 py-2";
  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <div className={style_str}>
          <button disabled={loading}>
            {loading ? "Processing..." : "Change RC/Local mode"}
          </button>
        </div>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
          <AlertDialogDescription>
            Warning: when the mode will be changed, the voltages should be 0. Do
            you already turn off the all channel?
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction>
            <button onClick={handleSubmit}>Continue</button>
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
};

export default RCButton;

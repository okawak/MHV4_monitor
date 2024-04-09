"use client";

import React, { useState } from "react";
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

interface RCProps {
  do_rc: boolean;
}

const RCButton: React.FC<RCProps> = ({ do_rc }) => {
  const [loading, setLoading] = useState(false);

  const handleSubmit = async () => {
    setLoading(true);
    console.log("input value: RC mode is >", do_rc);
    try {
      const response = await fetch(`${process.env.NEXT_PUBLIC_STATUS_ROUTE}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(!do_rc),
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

  if (do_rc) {
    style_str += " bg-green-700 text-primary-foreground hover:bg-green-700/80";
  } else {
    style_str += " bg-red-700 text-primary-foreground hover:bg-red-700/80";
  }
  style_str += " mx-1 h-8 px-4 py-2";
  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <div className={style_str}>
          <button onClick={handleSubmit} disabled={loading}>
            {loading ? "Processing..." : "Change RC/Local mode"}
          </button>
        </div>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
          <AlertDialogDescription>
            This action cannot be undone. This will permanently delete your
            account and remove your data from our servers.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction>Continue</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
};

export default RCButton;

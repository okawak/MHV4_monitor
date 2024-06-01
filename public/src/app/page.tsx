"use client";

import React, { useState, useEffect } from "react";

import { useMHV4Data } from "@/contexts/MHV4Context";
import ShowDate from "@/components/show-date";
import RCButton from "@/components/rc-button";
import PrintButton from "@/components/print-button";
import MHV4Table from "@/components/mhv4-table";
import OnoffButton from "@/components/onoff-button";
import ApplyButton from "@/components/apply-button";

const mhv4_descriptions = [
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
  ["---", "---"],
];

export default function Home() {
  const { voltageArray, isOnArray } = useMHV4Data();
  const [inputValues, setInputValues] = useState<number[]>([]);
  const [onoffStates, setOnoffStates] = useState<boolean[]>([]);

  useEffect(() => {
    setInputValues(new Array(voltageArray.length).fill(0));
  }, [voltageArray.length]);

  useEffect(() => {
    setOnoffStates([...isOnArray]);
  }, [isOnArray.length]);

  const handleValueChange = (newValue: number, index: number) => {
    const updatedValues = inputValues.map((value, i) =>
      i === index ? newValue : value,
    );
    setInputValues(updatedValues);
  };

  const handleStateChange = (state: boolean, index: number) => {
    const updatedStates = onoffStates.map((value, i) =>
      i === index ? state : value,
    );
    setOnoffStates(updatedStates);
  };

  return (
    <main>
      <h1 className="bg-gray-100 px-5 py-5 text-3xl font-bold">MHV4 monitor</h1>
      <ShowDate />
      <PrintButton />
      <RCButton />
      <MHV4Table
        userDescription={mhv4_descriptions}
        onCheckedChange={(state, index) => handleStateChange(state, index)}
        onValueChange={(newValue, index) => handleValueChange(newValue, index)}
      />
      <OnoffButton inputs={onoffStates} />
      <ApplyButton inputs={inputValues} />
    </main>
  );
}

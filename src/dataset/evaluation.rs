use std::{
    fmt::{Debug, Display},
    net::IpAddr,
};

#[allow(unused)]
pub trait EvaluationResult {
    type Item: Debug + Display;

    fn true_positives(&self) -> &[Self::Item];
    fn false_positives(&self) -> &[Self::Item];
    fn false_negatives(&self) -> &[Self::Item];

    fn total(&self) -> u64;

    fn true_positive_cnt(&self) -> u64 {
        self.true_positives().len() as u64
    }

    fn false_positive_cnt(&self) -> u64 {
        self.false_positives().len() as u64
    }

    fn false_negative_cnt(&self) -> u64 {
        self.false_negatives().len() as u64
    }

    fn true_negative_cnt(&self) -> u64 {
        self.total()
            - self.false_negative_cnt()
            - self.false_positive_cnt()
            - self.true_positive_cnt()
    }

    fn true_positive_rate(&self) -> f64 {
        let tp_ = self.true_positive_cnt() as f64;
        let fn_ = self.false_negative_cnt() as f64;
        tp_ / (tp_ + fn_)
    }

    fn false_positive_rate(&self) -> f64 {
        let tn_ = self.true_negative_cnt() as f64;
        let fp_ = self.false_positive_cnt() as f64;
        if tn_ + fp_ == 0. {
            0.
        } else {
            fp_ / (tn_ + fp_)
        }
    }

    fn precision(&self) -> f64 {
        let tp_ = self.true_positive_cnt() as f64;
        let fp_ = self.false_positive_cnt() as f64;
        tp_ / (tp_ + fp_)
    }

    fn recall(&self) -> f64 {
        self.true_positive_rate()
    }

    fn report(&self) {
        // alerts, false alerts and missed alerts
        println!(
            "TP({}): {:?}",
            self.true_positive_cnt(),
            self.true_positives()
        );
        println!(
            "FP({}): {:?}",
            self.false_positive_cnt(),
            self.false_positives()
        );
        println!(
            "FN({}): {:?}",
            self.false_negative_cnt(),
            self.false_negatives()
        );
        // Count statistics
        println!(
            "Count\tT\tF\nP\t{tp_}\t{fp_}\nN\t{tn_}\t{fn_}",
            tp_ = self.true_positive_cnt(),
            fp_ = self.false_positive_cnt(),
            fn_ = self.false_negative_cnt(),
            tn_ = self.true_negative_cnt(),
        );
        // Metrics
        println!(
            "TPR(recall) = {}, FPR = {}, precision = {}",
            self.true_positive_rate(),
            self.false_positive_rate(),
            self.precision()
        )
    }
}

pub struct ClientEvaluationResult {
    pub tp_: Vec<IpAddr>,
    pub fn_: Vec<IpAddr>,
    pub fp_: Vec<IpAddr>,
    pub total: u64,
}

impl EvaluationResult for ClientEvaluationResult {
    type Item = IpAddr;

    fn true_positives(&self) -> &[Self::Item] {
        &self.tp_
    }

    fn false_positives(&self) -> &[Self::Item] {
        &self.fp_
    }

    fn false_negatives(&self) -> &[Self::Item] {
        &self.fn_
    }

    fn total(&self) -> u64 {
        self.total
    }
}

pub struct DomainEvaluationResult {
    pub tp_: Vec<String>,
    pub fn_: Vec<String>,
    pub fp_: Vec<String>,
    pub total: u64,
}

impl EvaluationResult for DomainEvaluationResult {
    type Item = String;

    fn true_positives(&self) -> &[Self::Item] {
        &self.tp_
    }

    fn false_positives(&self) -> &[Self::Item] {
        &self.fp_
    }

    fn false_negatives(&self) -> &[Self::Item] {
        &self.fn_
    }

    fn total(&self) -> u64 {
        self.total
    }
}

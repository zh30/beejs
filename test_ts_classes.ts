// 测试类和接口
interface Person {
    name: string;
    age: number;
}

class Student {
    studentId: number;
    grade: string;

    constructor(id: number, grade: string) {
        this.studentId = id;
        this.grade = grade;
    }

    display(): void {
        console.log("Student ID:", this.studentId);
    }
}

const student = new Student(1001, "A");
student.display();
console.log("Grade:", student.grade);
